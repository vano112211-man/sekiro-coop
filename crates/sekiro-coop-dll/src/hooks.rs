//! Live hooks on the resolved native functions.
//!
//! Call [`install`] once on DLL attach with a resolved [`Natives`]
//! table.  Each successful install stores the trampoline (pointer to
//! the original implementation) in a module-static atomic, so detours
//! can tail-call it.
//!
//! **Calling conventions** (from the Cheat Engine table's
//! `executeCodeEx` usages, `SEKIRO_RAW.md` §11-12):
//!
//! - `SetFlag(this: EventFlagMan*, flag_id: u32, value: u8, unk: u32)`
//!    — x64 fastcall; MSVC `__fastcall` → Rust `extern "system"`.
//! - `ApplyEffect(entity: ChrIns*, speffect_id: u32)` — same ABI.

use core::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::collections::HashMap;
use std::sync::Mutex;

use once_cell::sync::Lazy;
use sekiro_sdk_bridge::events::BridgeEvent;
use sekiro_sdk_bridge::world::EventFlagEvent;
use sekiro_sdk_core::hook;
use sekiro_sdk_sys::natives::Natives;

/// Last observed `value` per `flag_id`.  Used for edge-triggered
/// emission so we don't broadcast the same flag-set to the peer every
/// tick (many flags — idol state, animation gates — are re-asserted
/// constantly even when nothing changed).
static FLAG_LAST_VALUE: Lazy<Mutex<HashMap<u32, u8>>> =
    Lazy::new(|| Mutex::new(HashMap::with_capacity(1024)));

// Original trampolines, stored as raw addresses for atomic safety.
static SETFLAG_ORIG: AtomicUsize = AtomicUsize::new(0);
static APPLY_EFFECT_ORIG: AtomicUsize = AtomicUsize::new(0);
static DELETE_EFFECT_ORIG: AtomicUsize = AtomicUsize::new(0);
static GIVE_ITEM_ORIG: AtomicUsize = AtomicUsize::new(0);
static ADD_EXPERIENCE_ORIG: AtomicUsize = AtomicUsize::new(0);
static WARP_BONFIRE_ORIG: AtomicUsize = AtomicUsize::new(0);

// The `this` pointers captured from the first detour hit — needed to
// call the trampolines with a valid context when *we* originate a call
// (from a remote BridgeEvent apply path).
pub static EVENT_FLAG_MAN_PTR: AtomicUsize = AtomicUsize::new(0);

// Call-count telemetry — visible via the overlay.
pub static SETFLAG_CALLS: AtomicU64 = AtomicU64::new(0);
pub static APPLY_EFFECT_CALLS: AtomicU64 = AtomicU64::new(0);
pub static DELETE_EFFECT_CALLS: AtomicU64 = AtomicU64::new(0);
pub static GIVE_ITEM_CALLS: AtomicU64 = AtomicU64::new(0);
pub static ADD_EXPERIENCE_CALLS: AtomicU64 = AtomicU64::new(0);
pub static WARP_BONFIRE_CALLS: AtomicU64 = AtomicU64::new(0);

type SetFlagFn = unsafe extern "system" fn(this: usize, flag_id: u32, value: u8, unk: u32) -> u8;
type ApplyEffectFn = unsafe extern "system" fn(entity: usize, speffect_id: u32) -> u64;
type DeleteEffectFn = unsafe extern "system" fn(effect_list: usize, speffect_id: u32) -> u64;
/// GiveItem(MapItemMan, itemList, unk3, unk4, unk5, unk6).  Source:
/// `ct_aa_scripts.json` "Spawn Item" — the `itemList` layout is:
///   +0x00 u32  items_count
///   +0x04 u16  id (real item id)
///   +0x06 u16  category (0=weapon 1=armor 2=accessory 3=goods)
///   +0x08 u32  quantity
///   +0x0C i32  durability
type GiveItemFn = unsafe extern "system" fn(
    this: usize,
    item_list: usize,
    unk3: u64,
    unk4: u64,
) -> u64;
/// AddExperience: `(gamedata_ptr, xp_amount)`.
type AddExperienceFn = unsafe extern "system" fn(this: usize, xp: u32) -> u64;
/// WarpNextStage_Bonfire(proxy=null, warp_id: u32, unk=0).  Source:
/// `ct_aa_scripts.json` "Warp".  Fires on idol rest / fast-travel.
type WarpBonfireFn = unsafe extern "system" fn(proxy: usize, warp_id: u32, unk: u32) -> u64;

/// SetFlag detour.  Logs every call at trace level (very high volume —
/// the game fires these on nearly every tick for idol flags, animation
/// state, etc.) and feeds a `BridgeEvent::EventFlagSet` to the
/// dispatcher so higher layers can observe.  Then tail-calls the
/// original.
extern "system" fn setflag_detour(this: usize, flag_id: u32, value: u8, unk: u32) -> u8 {
    SETFLAG_CALLS.fetch_add(1, Ordering::Relaxed);
    crate::puppet::on_game_thread();
    // Cache the EventFlagMan `this` pointer so remote-apply has
    // something valid to pass when it calls the trampoline.
    if EVENT_FLAG_MAN_PTR.load(Ordering::Relaxed) == 0 && this != 0 {
        EVENT_FLAG_MAN_PTR.store(this, Ordering::Relaxed);
    }
    tracing::trace!(flag_id, value, "SetFlag");

    if is_interesting_flag(flag_id) {
        // Edge-triggered: skip if the flag's value hasn't actually
        // changed since we last saw it.  Many flags are re-asserted
        // every frame by EMEVD and would otherwise flood the bridge.
        let changed = {
            let mut cache = FLAG_LAST_VALUE.lock().expect("flag cache poisoned");
            match cache.insert(flag_id, value) {
                Some(prev) => prev != value,
                None => true, // first time we've seen this flag
            }
        };
        if changed {
            tracing::debug!(flag_id, value, "SetFlag (interesting, edge)");
            if let Some(m) = crate::global() {
                let synced = m.world_bridge.should_sync(flag_id);
                m.dispatcher.emit(BridgeEvent::EventFlagSet(EventFlagEvent {
                    flag_id,
                    state: value != 0,
                    synced,
                }));
            }
        }
    }

    let orig = SETFLAG_ORIG.load(Ordering::Acquire);
    if orig == 0 {
        return 0;
    }
    let orig_fn: SetFlagFn = unsafe { core::mem::transmute(orig) };
    unsafe { orig_fn(this, flag_id, value, unk) }
}

/// ApplyEffect detour.  SpEffect applications are far rarer than flag
/// writes (seconds apart on average) — safe to log at info.  This is
/// the Phase-A exit criterion: "every SpEffect application appears in
/// log" (SPEC §10).
extern "system" fn applyeffect_detour(entity: usize, speffect_id: u32) -> u64 {
    APPLY_EFFECT_CALLS.fetch_add(1, Ordering::Relaxed);

    // Hook-supplied ChrIns pointer is known-good.  Register it keyed by
    // its `handle` (ChrIns+0x08) so remote peers, which only know
    // handles, can resolve to a local pointer.
    if entity >= 0x100_000 && entity < 0x0000_7fff_ffff_ffff {
        if let Some(m) = crate::global() {
            let handle = unsafe {
                sekiro_sdk_sys::live::handle_of(sekiro_sdk_sys::memory::RawPtr(
                    entity,
                ))
            };
            if handle != 0 {
                m.chrins_registry.lock().insert(handle, entity);
            }
        }
    }

    // Enrich the log with the SpEffect row's `effectEndurance` if we
    // can resolve the param table live.  Costs a single param lookup
    // per call — SpEffect applications are rare (~60/min baseline).
    let endurance = crate::global().and_then(|m| {
        m.base_addrs.as_ref().map(|_| ()).and(
            sekiro_sdk_sys::memory::find_current_module("sekiro.exe")
                .ok()
                .and_then(|module| unsafe {
                    sekiro_sdk_sys::paramrepo::speffect_param_row(
                        sekiro_sdk_sys::paramrepo::SOLO_PARAM_REPOSITORY_RVA_V1_06,
                        module.base,
                        speffect_id,
                    )
                    .map(|row| row.offset(0x8).read::<f32>())
                }),
        )
    });

    tracing::info!(
        entity = format!("{entity:#x}"),
        speffect_id,
        endurance = ?endurance,
        "ApplyEffect"
    );
    if let Some(m) = crate::global() {
        m.dispatcher.emit(BridgeEvent::SpEffectApplied {
            entity: entity as u64,
            id: speffect_id as i32,
        });
    }
    let orig = APPLY_EFFECT_ORIG.load(Ordering::Acquire);
    if orig == 0 {
        return 0;
    }
    let orig_fn: ApplyEffectFn = unsafe { core::mem::transmute(orig) };
    unsafe { orig_fn(entity, speffect_id) }
}

/// SpecialEffectDeleteEffect detour.  Pairs with ApplyEffect; together
/// they let us observe SpEffect lifetime on every character.  Args:
/// `(effect_list_ptr, speffect_id)` — per `SEKIRO_RAW.md §12`.
extern "system" fn delete_effect_detour(effect_list: usize, speffect_id: u32) -> u64 {
    DELETE_EFFECT_CALLS.fetch_add(1, Ordering::Relaxed);
    tracing::info!(
        effect_list = format!("{effect_list:#x}"),
        speffect_id,
        "DeleteEffect"
    );
    if let Some(m) = crate::global() {
        m.dispatcher.emit(BridgeEvent::SpEffectRemoved {
            entity: effect_list as u64,
            id: speffect_id as i32,
        });
    }
    let orig = DELETE_EFFECT_ORIG.load(Ordering::Acquire);
    if orig == 0 {
        return 0;
    }
    let orig_fn: DeleteEffectFn = unsafe { core::mem::transmute(orig) };
    unsafe { orig_fn(effect_list, speffect_id) }
}

/// GiveItem detour.  Observes every item granted to the player.
extern "system" fn give_item_detour(
    this: usize,
    item_list: usize,
    unk3: u64,
    unk4: u64,
) -> u64 {
    GIVE_ITEM_CALLS.fetch_add(1, Ordering::Relaxed);

    // Dereference the itemList struct to extract the real fields.
    // Defensive: if item_list is near-null, skip the decode.
    let (count, id, category, quantity) = if item_list >= 0x1_0000 {
        unsafe {
            let base = item_list as *const u8;
            let items_count = core::ptr::read(base as *const u32);
            let id = core::ptr::read(base.add(0x4) as *const u16);
            let category = core::ptr::read(base.add(0x6) as *const u16);
            let quantity = core::ptr::read(base.add(0x8) as *const u32);
            (items_count, id, category, quantity)
        }
    } else {
        (0, 0, 0, 0)
    };

    // Live lookup: is this a known EquipParamGoods row?  Non-null row
    // confirms `category == goods`.  Just a sanity check; we don't
    // decode name (names live in FMGs, out-of-proc).
    let goods_row = if category == 3 {
        crate::global()
            .and_then(|_| {
                sekiro_sdk_sys::memory::find_current_module("sekiro.exe").ok()
            })
            .and_then(|module| unsafe {
                sekiro_sdk_sys::paramrepo::open_param(
                    sekiro_sdk_sys::paramrepo::SOLO_PARAM_REPOSITORY_RVA_V1_06,
                    module.base,
                    sekiro_sdk_sys::paramrepo::ParamKind::EquipParamGoods,
                )
                .and_then(|t| t.row(id as u32))
            })
    } else {
        None
    };

    tracing::info!(
        this = format!("{this:#x}"),
        item_list = format!("{item_list:#x}"),
        count,
        id,
        category,
        quantity,
        goods_row_found = goods_row.is_some(),
        "GiveItem"
    );
    if let Some(m) = crate::global() {
        m.dispatcher.emit(BridgeEvent::ItemReceived {
            item_id: id as u32,
            count: quantity,
        });
    }
    let orig = GIVE_ITEM_ORIG.load(Ordering::Acquire);
    if orig == 0 {
        return 0;
    }
    let orig_fn: GiveItemFn = unsafe { core::mem::transmute(orig) };
    unsafe { orig_fn(this, item_list, unk3, unk4) }
}

extern "system" fn warp_bonfire_detour(proxy: usize, warp_id: u32, unk: u32) -> u64 {
    WARP_BONFIRE_CALLS.fetch_add(1, Ordering::Relaxed);
    tracing::info!(
        proxy = format!("{proxy:#x}"),
        warp_id,
        unk,
        "WarpNextStage_Bonfire"
    );
    let orig = WARP_BONFIRE_ORIG.load(Ordering::Acquire);
    if orig == 0 {
        return 0;
    }
    let orig_fn: WarpBonfireFn = unsafe { core::mem::transmute(orig) };
    unsafe { orig_fn(proxy, warp_id, unk) }
}

/// AddExperience detour.
extern "system" fn add_experience_detour(this: usize, xp: u32) -> u64 {
    ADD_EXPERIENCE_CALLS.fetch_add(1, Ordering::Relaxed);
    tracing::info!(
        this = format!("{this:#x}"),
        xp,
        "AddExperience"
    );
    if let Some(m) = crate::global() {
        m.dispatcher
            .emit(BridgeEvent::ExperienceGained { amount: xp });
    }
    let orig = ADD_EXPERIENCE_ORIG.load(Ordering::Acquire);
    if orig == 0 {
        return 0;
    }
    let orig_fn: AddExperienceFn = unsafe { core::mem::transmute(orig) };
    unsafe { orig_fn(this, xp) }
}

/// Interesting-flag classifier.  Kept small + fast; detour calls this
/// on every SetFlag hit.
fn is_interesting_flag(id: u32) -> bool {
    // Demon Bell toggle
    if id == 9830 {
        return true;
    }
    // Charmless toggle
    if id == 6911 {
        return true;
    }
    // Idol flag range (SEKIRO_OFFSETS_ADDENDUM.md §C):
    // idols live in 11_000_000..=12_920_005.
    if (11_000_000..=12_920_005).contains(&id) {
        return true;
    }
    false
}

/// Installation result for one hook.
#[derive(Debug, Clone)]
pub struct HookInstall {
    pub name: &'static str,
    pub target: Option<usize>,
    pub installed: bool,
    pub error: Option<String>,
}

/// Install every supported hook.  Never panics; hooks that fail to
/// resolve or create are reported in the returned `Vec` so the DLL
/// can log them and carry on.
pub fn install(natives: &Natives) -> Vec<HookInstall> {
    let mut out = Vec::new();

    // SetFlag
    if let Some(addr) = natives.functions.set_flag {
        out.push(install_one("SetFlag", addr, setflag_detour as usize, &SETFLAG_ORIG));
    } else {
        out.push(HookInstall {
            name: "SetFlag",
            target: None,
            installed: false,
            error: Some("AOB did not resolve".into()),
        });
    }

    // ApplyEffect
    if let Some(addr) = natives.functions.apply_effect {
        out.push(install_one(
            "ApplyEffect",
            addr,
            applyeffect_detour as usize,
            &APPLY_EFFECT_ORIG,
        ));
    } else {
        out.push(HookInstall {
            name: "ApplyEffect",
            target: None,
            installed: false,
            error: Some("AOB did not resolve".into()),
        });
    }

    // SpecialEffectDeleteEffect
    if let Some(addr) = natives.functions.special_effect_delete_effect {
        out.push(install_one(
            "DeleteEffect",
            addr,
            delete_effect_detour as usize,
            &DELETE_EFFECT_ORIG,
        ));
    } else {
        out.push(HookInstall {
            name: "DeleteEffect",
            target: None,
            installed: false,
            error: Some("AOB did not resolve".into()),
        });
    }

    // GiveItem
    if let Some(addr) = natives.functions.give_item {
        out.push(install_one(
            "GiveItem",
            addr,
            give_item_detour as usize,
            &GIVE_ITEM_ORIG,
        ));
    } else {
        out.push(HookInstall {
            name: "GiveItem",
            target: None,
            installed: false,
            error: Some("AOB did not resolve".into()),
        });
    }

    // AddExperience
    if let Some(addr) = natives.functions.add_experience {
        out.push(install_one(
            "AddExperience",
            addr,
            add_experience_detour as usize,
            &ADD_EXPERIENCE_ORIG,
        ));
    } else {
        out.push(HookInstall {
            name: "AddExperience",
            target: None,
            installed: false,
            error: Some("AOB did not resolve".into()),
        });
    }

    // WarpNextStage_Bonfire
    if let Some(addr) = natives.functions.warp_next_stage_bonfire {
        out.push(install_one(
            "WarpBonfire",
            addr,
            warp_bonfire_detour as usize,
            &WARP_BONFIRE_ORIG,
        ));
    } else {
        out.push(HookInstall {
            name: "WarpBonfire",
            target: None,
            installed: false,
            error: Some("AOB did not resolve".into()),
        });
    }

    out
}

fn install_one(
    name: &'static str,
    target: usize,
    detour: usize,
    orig_slot: &AtomicUsize,
) -> HookInstall {
    match hook::create_hook(target, detour) {
        Ok((_handle, tramp)) => {
            orig_slot.store(tramp as usize, Ordering::Release);
            tracing::info!(name, target = format!("{target:#x}"), "hook installed");
            HookInstall {
                name,
                target: Some(target),
                installed: true,
                error: None,
            }
        }
        Err(e) => {
            let msg = e.to_string();
            tracing::warn!(name, target = format!("{target:#x}"), err = %msg, "hook failed");
            HookInstall {
                name,
                target: Some(target),
                installed: false,
                error: Some(msg),
            }
        }
    }
}

/// Snapshot of the hook telemetry — for the overlay / periodic logs.
#[derive(Debug, Clone, Copy, Default)]
pub struct HookStats {
    pub setflag_calls: u64,
    pub apply_effect_calls: u64,
    pub delete_effect_calls: u64,
    pub give_item_calls: u64,
    pub add_experience_calls: u64,
    pub warp_bonfire_calls: u64,
}

pub fn stats() -> HookStats {
    HookStats {
        setflag_calls: SETFLAG_CALLS.load(Ordering::Relaxed),
        apply_effect_calls: APPLY_EFFECT_CALLS.load(Ordering::Relaxed),
        delete_effect_calls: DELETE_EFFECT_CALLS.load(Ordering::Relaxed),
        give_item_calls: GIVE_ITEM_CALLS.load(Ordering::Relaxed),
        add_experience_calls: ADD_EXPERIENCE_CALLS.load(Ordering::Relaxed),
        warp_bonfire_calls: WARP_BONFIRE_CALLS.load(Ordering::Relaxed),
    }
}

// ---------------------------------------------------------------------
//  Remote event application.
//
//  When a BridgeEvent arrives from the peer, we can apply it locally by
//  calling the native function's *trampoline* directly.  Trampolines
//  are the pre-hook prologue bytes MinHook relocated; calling them
//  bypasses our detour, so there's no rebroadcast → no feedback loop.
//
//  Safety gate: off by default.  Set `SEKIRO_COOP_APPLY_REMOTE=1` in
//  the host environment before launch to enable.  Without the gate,
//  events are log-only — preserves user saves during development.
// ---------------------------------------------------------------------

/// True iff the apply gate is open (env var set).
fn apply_gate_open() -> bool {
    std::env::var("SEKIRO_COOP_APPLY_REMOTE")
        .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
        .unwrap_or(false)
}

/// Apply one remote BridgeEvent to the local game state.  No-op when
/// the gate is closed or the trampoline isn't captured.  Designed to
/// be safe to call from the tick thread.
pub fn apply_remote_event(ev: &sekiro_sdk_bridge::events::BridgeEvent) {
    use sekiro_sdk_bridge::events::BridgeEvent;

    if !apply_gate_open() {
        tracing::trace!(?ev, "remote event (gate closed, not applied)");
        return;
    }

    match ev {
        BridgeEvent::EventFlagSet(e) => {
            let this = EVENT_FLAG_MAN_PTR.load(Ordering::Acquire);
            let tramp = SETFLAG_ORIG.load(Ordering::Acquire);
            if this == 0 || tramp == 0 {
                tracing::debug!(flag_id = e.flag_id, "skip remote flag: EventFlagMan not yet captured");
                return;
            }
            tracing::info!(
                flag_id = e.flag_id,
                state = e.state,
                "applying remote SetFlag"
            );
            let orig: SetFlagFn = unsafe { core::mem::transmute(tramp) };
            unsafe { orig(this, e.flag_id, e.state as u8, 0) };
        }
        BridgeEvent::SpEffectApplied { entity: _, id } => {
            let tramp = APPLY_EFFECT_ORIG.load(Ordering::Acquire);
            if tramp == 0 {
                tracing::debug!(speffect_id = id, "skip remote speffect: trampoline unset");
                return;
            }
            // Apply to LOCAL player ChrIns — not the remote's entity
            // pointer (which is meaningless in our address space).
            let m = match crate::global() { Some(m) => m, None => return };
            let module = match sekiro_sdk_sys::memory::find_current_module("sekiro.exe") {
                Ok(m) => m,
                Err(_) => return,
            };
            let Some(addrs) = m.base_addrs.as_ref() else { return };
            let chr_sym = module.base + addrs.player_position;
            // WorldChrMan → +0x88 (Hero ChrIns pointer)
            let hero: usize = unsafe {
                let repo: usize = (chr_sym as *const usize).read();
                if repo == 0 { return; }
                ((repo + 0x88) as *const usize).read()
            };
            if hero == 0 {
                return;
            }
            tracing::info!(
                speffect_id = id,
                entity = format!("{hero:#x}"),
                "applying remote SpEffect to local player"
            );
            let orig: ApplyEffectFn = unsafe { core::mem::transmute(tramp) };
            unsafe { orig(hero, *id as u32) };
        }
        BridgeEvent::SpEffectRemoved { entity: _, id: _ } => {
            // Skipped: SpEffect-remove needs the effect-list ptr from a
            // specific target, not a simple handle.  Defer.
            tracing::trace!(?ev, "remote SpEffectRemoved ignored (needs effect-list lookup)");
        }
        BridgeEvent::ItemReceived { .. }
        | BridgeEvent::ExperienceGained { .. } => {
            // Per-player state; user explicitly said not needed for MP.
            tracing::trace!(?ev, "remote item/xp event ignored (per-player)");
        }
        _ => {
            tracing::trace!(?ev, "remote event type not applied");
        }
    }
}
