//! Puppet probe v2 — diagnostic for the "remote player is not rendered" blocker.
//!
//! Enabled only with `SEKIRO_COOP_PUPPET_PROBE=1`.
//!
//! The original puppet experiment wrote an NPC's physics-module position
//! (`ChrIns+0x1ff8 → +0x68 → +0x80`) from the 60 Hz fallback ticker thread and
//! read it back immediately, so "actual == target" said nothing about whether
//! the game overwrote it during its own frame. This probe:
//!
//! 1. Surveys the registry once per second (`puppet survey`): handle, c-id,
//!    team, distance to the Hero, animation — to pick a visible character.
//! 2. Pins one target (nearest team-6 character within 12 m, or the c-id in
//!    `SEKIRO_COOP_PUPPET_CHAR`) within 15 m and holds it
//!    `SEKIRO_COOP_PUPPET_OFFSET` metres (default 3) ahead of the Hero along
//!    the Hero's last movement direction. After a reload the same handle is
//!    re-pinned once the registry has its new pointer.
//! 2b. Writes `SEKIRO_COOP_PUPPET_TEAM` (default 2 = WhitePhantom, DS3
//!    numbering; -1 = off) to the target's team byte (ChrIns+0x74).
//! 3. Writes from the game thread inside the SetFlag detour (default), or from
//!    the ticker thread with `SEKIRO_COOP_PUPPET_WRITER=ticker`, and counts how
//!    often the game changed the value between two of our writes.
//!
//! Every pointer hop is checked with `VirtualQuery` first, and the target's
//! handle is re-read before each write, so a despawned target is dropped
//! instead of dereferenced.

use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};

use once_cell::sync::{Lazy, OnceCell};
use parking_lot::Mutex;

use sekiro_sdk_sys::memory::RawPtr;

pub static ENABLED: Lazy<bool> = Lazy::new(|| {
    std::env::var("SEKIRO_COOP_PUPPET_PROBE").map(|v| v == "1").unwrap_or(false)
});
static PIN_CHAR: Lazy<Option<u32>> = Lazy::new(|| {
    std::env::var("SEKIRO_COOP_PUPPET_CHAR").ok().and_then(|v| v.trim().parse().ok())
});
static OFFSET_M: Lazy<f32> = Lazy::new(|| {
    std::env::var("SEKIRO_COOP_PUPPET_OFFSET")
        .ok()
        .and_then(|v| v.trim().parse().ok())
        .unwrap_or(3.0)
});
/// Team byte written to the puppet (ChrIns+0x74). Default 2 = WhitePhantom
/// (DS3 numbering; the Hero is 1). `SEKIRO_COOP_PUPPET_TEAM=-1` disables.
static TEAM: Lazy<Option<u8>> = Lazy::new(|| {
    match std::env::var("SEKIRO_COOP_PUPPET_TEAM").ok().and_then(|v| v.trim().parse::<i32>().ok()) {
        Some(v) if v < 0 => None,
        Some(v) => Some(v as u8),
        None => Some(2),
    }
});
const PICK_RADIUS_M: f32 = 15.0;
/// Test aid: set the game's global debug flag "all no update AI"
/// (`debug_flags + 13`) so every enemy, the puppet included, stops thinking.
/// Not a co-op solution (it freezes all enemies); it isolates "is the puppet's
/// own AI what attacks" from everything else.
static FREEZE_AI: Lazy<bool> = Lazy::new(|| {
    std::env::var("SEKIRO_COOP_PUPPET_FREEZE_AI").map(|v| v == "1").unwrap_or(false)
});
static FREEZE_AI_BEFORE: Mutex<Option<u8>> = Mutex::new(None);
/// Extra ChrIns-relative offsets that hold a live copy of the position
/// (found by the mirror scan, run 7: +0x1050 and +0x1060 in both scans).
/// `SEKIRO_COOP_PUPPET_MIRRORS=0x1050,0x1060` (default) or `none`.
static MIRRORS: Lazy<Vec<usize>> = Lazy::new(|| {
    match std::env::var("SEKIRO_COOP_PUPPET_MIRRORS") {
        Ok(v) if v.trim() == "none" => Vec::new(),
        Ok(v) => v
            .split(',')
            .filter_map(|x| usize::from_str_radix(x.trim().trim_start_matches("0x"), 16).ok())
            .take(4)
            .collect(),
        Err(_) => Vec::new(),
    }
});
/// `warp` (default): move the puppet with the game's own
/// ChrIns::Warp (vtable +0x6A0, found in EventChrEntity::Warp). `raw`: the
/// old physics-module position write.
static MODE_RAW: Lazy<bool> = Lazy::new(|| {
    std::env::var("SEKIRO_COOP_PUPPET_MODE").map(|v| v == "raw").unwrap_or(false)
});
/// Hybrid (default): native warp at most every `SEKIRO_COOP_PUPPET_WARP_MS`
/// (default 500) or when the puppet is more than 2 m from where it should be;
/// raw physics-position steps every 8 ms in between. Warping every frame made
/// the puppet flicker (run 9). Default 50 ms since run 16: with the animation
/// reset in place it is smooth and the game plays the puppet's own walk/run cycle.
static WARP_MS: Lazy<u64> = Lazy::new(|| {
    std::env::var("SEKIRO_COOP_PUPPET_WARP_MS").ok().and_then(|v| v.trim().parse().ok()).unwrap_or(50)
});
/// Where the puppet goes. `offset` (default): `SEKIRO_COOP_PUPPET_OFFSET` m
/// ahead of the Hero. `net`: the remote player's position from the latest
/// PlayerSnapshot; the puppet holds still if no snapshot arrived for 1 s.
static SOURCE_NET: Lazy<bool> = Lazy::new(|| {
    std::env::var("SEKIRO_COOP_PUPPET_SOURCE").map(|v| v == "net").unwrap_or(false)
});
static REMOTE_SEEN_AT: Mutex<Option<std::time::Instant>> = Mutex::new(None);
static REMOTE_FRESH_STEPS: AtomicU64 = AtomicU64::new(0);
static REMOTE_STALE_STEPS: AtomicU64 = AtomicU64::new(0);

/// Called by the packet handler whenever a PlayerSnapshot arrives.
pub fn note_remote_snapshot() {
    *REMOTE_SEEN_AT.lock() = Some(std::time::Instant::now());
}

/// Remote player's facing from the latest fresh snapshot.
fn remote_yaw() -> Option<f32> {
    let fresh = REMOTE_SEEN_AT
        .lock()
        .map(|t| t.elapsed() < std::time::Duration::from_secs(1))
        .unwrap_or(false);
    if !fresh {
        return None;
    }
    let m = crate::global()?;
    let snap = m.remote_player.lock().clone()?;
    Some(snap.yaw).filter(|y| y.is_finite())
}

fn remote_target() -> Option<[f32; 3]> {
    let fresh = REMOTE_SEEN_AT
        .lock()
        .map(|t| t.elapsed() < std::time::Duration::from_secs(1))
        .unwrap_or(false);
    if !fresh {
        return None;
    }
    let m = crate::global()?;
    let snap = m.remote_player.lock().clone()?;
    Some(snap.position)
}
static LAST_WARP_AT: Mutex<Option<std::time::Instant>> = Mutex::new(None);
static RAW_STEPS: AtomicU64 = AtomicU64::new(0);
/// Turn the puppet's own logic (AI) off: ChrCmd_EnableLogic clears bit 0 of
/// `[ChrIns+0x50]+0x190`. Default on; `SEKIRO_COOP_PUPPET_LOGIC_OFF=0` keeps it.
static LOGIC_OFF: Lazy<bool> = Lazy::new(|| {
    std::env::var("SEKIRO_COOP_PUPPET_LOGIC_OFF").map(|v| v != "0").unwrap_or(true)
});
static SCAN: Lazy<bool> = Lazy::new(|| {
    std::env::var("SEKIRO_COOP_PUPPET_SCAN").map(|v| v == "1").unwrap_or(false)
});
/// Sekiro 1.06 (exe SHA-256 637aca52…, PE stamp 0x5fa3e066) RVAs, read from a
/// memory image: ChrIns/EnemyIns/PlayerIns vtable slot +0x6A0 → 0x9f81f0.
const RVA_CHR_WARP: usize = 0x9f81f0;
const VT_SLOT_WARP: usize = 0x6A0;
type ChrWarpFn = unsafe extern "system" fn(chr: usize, pos: *const [f32; 4], rot: *const [f32; 4], a: u8, b: u8);
/// ChrCmd_ChrResetAnimation(false) → 0x9f4c60(chr): resets the animation
/// module `[ChrIns+0x1ff8]+0x28` (stops the puppet's run cycle and its root
/// motion, which moved it between our writes ~80% of the time in run 13).
const RVA_CHR_RESET_ANIM: usize = 0x9f4c60;
const RESET_ANIM_BYTES: [u8; 14] =
    [0x48, 0x8b, 0x81, 0xf8, 0x1f, 0x00, 0x00, 0x48, 0x8b, 0x48, 0x28, 0x48, 0x85, 0xc9];
type ChrResetAnimFn = unsafe extern "system" fn(chr: usize);

/// Script command ForcePlayAnimation, offline (1.06): `ComCmd_ForcePlayAnimation`
/// → `EventChrEntity::vt[+0x38]` (0x750e00) → `0xbd84c0(module, anim_id)` with
/// `module = [[ChrIns+0x1ff8]+0x58]`. It stores the id at module+0x10 and in
/// the owner's request slot; the game starts the animation on its next update.
const RVA_FORCE_PLAY_ANIM: usize = 0xbd84c0;
const FORCE_PLAY_ANIM_BYTES: [u8; 11] = [0x40, 0x53, 0x48, 0x83, 0xec, 0x20, 0x8b, 0xda, 0x89, 0x51, 0x10];
type ForcePlayAnimFn = unsafe extern "system" fn(module: usize, anim_id: u32);

/// Animation the puppet plays when the remote player starts an action.
/// `SEKIRO_COOP_PUPPET_ACTION_ANIM` (default 3000, an enemy attack seen in run 8;
/// 0 = off).
static ACTION_ANIM: Lazy<u32> = Lazy::new(|| {
    std::env::var("SEKIRO_COOP_PUPPET_ACTION_ANIM").ok().and_then(|v| v.parse().ok()).unwrap_or(3000)
});
static LAST_FORCE_AT: Mutex<Option<std::time::Instant>> = Mutex::new(None);
static FORCES: AtomicU64 = AtomicU64::new(0);
static FORCE_REFUSED: AtomicU64 = AtomicU64::new(0);
static RESETS_HELD: AtomicU64 = AtomicU64::new(0);
/// Remote animation tracking: (current id, since), and the id it shows at rest.
static REMOTE_ANIM: Mutex<Option<(u32, std::time::Instant)>> = Mutex::new(None);
static REST_ANIM: Mutex<Option<u32>> = Mutex::new(None);
/// Transitions to print from the ticker: (ms since start, from, to, moving, forced).
static ANIM_EVENTS: Mutex<Vec<(u128, u32, u32, bool, bool)>> = Mutex::new(Vec::new());
static ANIM_EVENTS_LOGGED: AtomicU64 = AtomicU64::new(0);
static T0: Lazy<std::time::Instant> = Lazy::new(std::time::Instant::now);
/// After a force: check the puppet's own animation a little later.
static FORCE_CHECK: Mutex<Option<(std::time::Instant, u32)>> = Mutex::new(None);
static RESET_ANIM: Lazy<bool> = Lazy::new(|| {
    std::env::var("SEKIRO_COOP_PUPPET_RESET_ANIM").map(|v| v != "0").unwrap_or(true)
});
static LAST_RESET_AT: Mutex<Option<std::time::Instant>> = Mutex::new(None);
static ANIM_RESETS: AtomicU64 = AtomicU64::new(0);
/// Stop detection for the target: last position it moved from and when.
static MOVE_TRACK: Mutex<Option<([f32; 3], std::time::Instant)>> = Mutex::new(None);
static TARGET_MOVING: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
static STOP_RESETS: AtomicU64 = AtomicU64::new(0);
static ANIM_RESET_REFUSED: AtomicU64 = AtomicU64::new(0);
/// Fraction of the remaining distance covered per 8 ms step (net source).
static SMOOTH: Lazy<f32> = Lazy::new(|| {
    std::env::var("SEKIRO_COOP_PUPPET_SMOOTH").ok().and_then(|v| v.trim().parse().ok()).unwrap_or(0.35)
});
/// Yaw (radians) the puppet should face: from the target's movement.
static YAW: Mutex<Option<f32>> = Mutex::new(None);
static YAW_TARGET: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);
static HAVE_YAW_TARGET: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
static LAST_TARGET: Mutex<Option<[f32; 3]>> = Mutex::new(None);
static YAW_OFFSET: Lazy<f32> = Lazy::new(|| {
    std::env::var("SEKIRO_COOP_PUPPET_YAW_OFFSET_DEG")
        .ok()
        .and_then(|v| v.trim().parse::<f32>().ok())
        .unwrap_or(0.0)
        .to_radians()
});
static WARPS: AtomicU64 = AtomicU64::new(0);
static IN_STEP: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
static WARP_REFUSED: AtomicU64 = AtomicU64::new(0);
static LOGIC_WRITES: AtomicU64 = AtomicU64::new(0);
static MIRROR_OVERWRITTEN: [AtomicU64; 4] =
    [AtomicU64::new(0), AtomicU64::new(0), AtomicU64::new(0), AtomicU64::new(0)];
static WRITER_TICKER: Lazy<bool> = Lazy::new(|| {
    std::env::var("SEKIRO_COOP_PUPPET_WRITER").map(|v| v == "ticker").unwrap_or(false)
});

/// `sekiro.exe` module base, stored on attach.
pub static MODULE_BASE: AtomicUsize = AtomicUsize::new(0);

#[derive(Clone, Copy, Debug)]
struct Target {
    handle: u32,
    ptr: usize,
    char_id: u32,
}

static TARGET: Mutex<Option<Target>> = Mutex::new(None);
static LAST_WRITTEN: Mutex<Option<[f32; 3]>> = Mutex::new(None);
static LAST_SEEN: Mutex<Option<[f32; 3]>> = Mutex::new(None);

/// Handle of the last target, kept after it goes stale so we can re-pin the
/// same character once the registry has its new pointer.
static LAST_HANDLE: Mutex<Option<(u32, u32)>> = Mutex::new(None);
/// Hero heading on the ground plane (x, z), from movement.
static HEADING: Mutex<([f32; 2], Option<[f32; 3]>)> = Mutex::new(([1.0, 0.0], None));
static TEAM_BEFORE: Mutex<Option<u8>> = Mutex::new(None);
static TEAM_WRITES: AtomicU64 = AtomicU64::new(0);
static REPINS: AtomicU64 = AtomicU64::new(0);
static RESCUED: AtomicU64 = AtomicU64::new(0);
/// Frame at which the current target was pinned (for scan scheduling).
static PINNED_AT: AtomicU64 = AtomicU64::new(0);
/// While non-zero, writes are paused until this frame (lets every copy of
/// the position settle to the game's own value before a mirror scan).
static PAUSE_UNTIL: AtomicU64 = AtomicU64::new(0);
static CUR_FRAME: AtomicU64 = AtomicU64::new(0);
static SCANS_DONE: AtomicU64 = AtomicU64::new(0);
static LAST_WRITE_AT: Mutex<Option<std::time::Instant>> = Mutex::new(None);
static WRITES: AtomicU64 = AtomicU64::new(0);
static OVERWRITTEN: AtomicU64 = AtomicU64::new(0);
static STALE: AtomicU64 = AtomicU64::new(0);
static WRITER_TID: OnceCell<String> = OnceCell::new();
static TICKER_TID: OnceCell<String> = OnceCell::new();

// ---------------------------------------------------------------- memory

#[cfg(windows)]
fn readable(addr: usize, len: usize) -> bool {
    use windows::Win32::System::Memory::{
        VirtualQuery, MEMORY_BASIC_INFORMATION, MEM_COMMIT, PAGE_GUARD, PAGE_NOACCESS,
    };
    if addr < 0x10000 {
        return false;
    }
    let mut mbi = MEMORY_BASIC_INFORMATION::default();
    let n = unsafe {
        VirtualQuery(
            Some(addr as *const core::ffi::c_void),
            &mut mbi,
            core::mem::size_of::<MEMORY_BASIC_INFORMATION>(),
        )
    };
    if n == 0 || mbi.State != MEM_COMMIT {
        return false;
    }
    if (mbi.Protect.0 & (PAGE_NOACCESS.0 | PAGE_GUARD.0)) != 0 || mbi.Protect.0 == 0 {
        return false;
    }
    let end = mbi.BaseAddress as usize + mbi.RegionSize;
    addr + len <= end
}

#[cfg(not(windows))]
fn readable(_addr: usize, _len: usize) -> bool {
    false
}

unsafe fn read_checked<T: Copy>(addr: usize) -> Option<T> {
    if readable(addr, core::mem::size_of::<T>()) {
        Some(RawPtr(addr).read::<T>())
    } else {
        None
    }
}

unsafe fn read_ptr(addr: usize) -> Option<usize> {
    read_checked::<usize>(addr).filter(|p| *p >= 0x10000)
}

/// True if `chr` still looks like a live ChrIns registered under `handle`:
/// the handle at +0x08 matches and every pointer hop that
/// `live::chrins_snapshot` / `chrins_write_hp` follow is mapped. Used to
/// prune the registry: after death/reload the game frees and re-creates
/// characters, and the stale pointers crashed `chrins_snapshot`
/// (run 11: AV at sekiro_coop.dll+0xac37c reading `[ChrIns+0x1ff8]+0x18`).
pub fn chrins_alive(chr: usize, handle: u32) -> bool {
    unsafe {
        if read_checked::<u32>(chr + 0x08) != Some(handle) {
            return false;
        }
        if !readable(chr, 0x80) {
            return false;
        }
        let Some(sm) = read_ptr(chr + 0x1ff8) else { return false };
        if !readable(sm, 0x70) {
            return false;
        }
        for (hop, len) in [(0x10usize, 0x24usize), (0x18, 0x150), (0x68, 0x90)] {
            match read_checked::<usize>(sm + hop) {
                Some(0) => {}
                Some(p) if p >= 0x10000 && readable(p, len) => {}
                _ => return false,
            }
        }
        true
    }
}

/// Drop registry entries that are no longer live ChrIns.
pub fn prune_registry(m: &crate::Mod) -> usize {
    let mut reg = m.chrins_registry.lock();
    let before = reg.len();
    reg.retain(|h, p| chrins_alive(*p, *h));
    before - reg.len()
}

/// Address of the physics-module position: `ChrIns+0x1ff8 → +0x68 → +0x80`.
unsafe fn phys_pos_addr(chrins: usize) -> Option<usize> {
    let sm = read_ptr(chrins + 0x1ff8)?;
    let phys = read_ptr(sm + 0x68)?;
    let p = phys + 0x80;
    readable(p, 12).then_some(p)
}

unsafe fn anim_of(chrins: usize) -> Option<u32> {
    let sm = read_ptr(chrins + 0x1ff8)?;
    let sub = read_ptr(sm + 0x10)?;
    read_checked::<u32>(sub + 0x20)
}

fn dist(a: [f32; 3], b: [f32; 3]) -> f32 {
    let (dx, dy, dz) = (a[0] - b[0], a[1] - b[1], a[2] - b[2]);
    (dx * dx + dy * dy + dz * dz).sqrt()
}

fn hero_pos() -> Option<[f32; 3]> {
    let m = crate::global()?;
    let addrs = m.base_addrs.as_ref()?;
    let base = MODULE_BASE.load(Ordering::Relaxed);
    if base == 0 {
        return None;
    }
    unsafe { sekiro_sdk_sys::live::player_position_xyz(addrs, base) }
}

/// Unit vector of the Hero's last movement on the ground plane (x, z). Only
/// updated after 0.3 m of travel so jitter does not spin it.
fn heading_update(hero: [f32; 3]) -> [f32; 2] {
    let mut g = HEADING.lock();
    match g.1 {
        None => g.1 = Some(hero),
        Some(prev) => {
            let (dx, dz) = (hero[0] - prev[0], hero[2] - prev[2]);
            let len = (dx * dx + dz * dz).sqrt();
            if len > 0.3 {
                g.0 = [dx / len, dz / len];
                g.1 = Some(hero);
            }
        }
    }
    g.0
}

/// Call ChrIns::Warp(pos, rot, 1, 0) on the game thread, but only after
/// checking that the object's vtable slot +0x6A0 points at the 1.06 function.
unsafe fn native_warp(chr: usize, pos: [f32; 3], rot: [f32; 4]) -> bool {
    let base = MODULE_BASE.load(Ordering::Relaxed);
    let Some(vt) = read_ptr(chr) else { return false };
    let Some(f) = read_checked::<usize>(vt + VT_SLOT_WARP) else { return false };
    if base == 0 || f != base + RVA_CHR_WARP {
        WARP_REFUSED.fetch_add(1, Ordering::Relaxed);
        return false;
    }
    let p: [f32; 4] = [pos[0], pos[1], pos[2], 1.0];
    let warp: ChrWarpFn = core::mem::transmute(f);
    warp(chr, &p, &rot, 1, 0);
    WARPS.fetch_add(1, Ordering::Relaxed);
    true
}

/// Call the native animation reset once its first bytes match 1.06.
unsafe fn reset_anim(chr: usize) -> bool {
    let base = MODULE_BASE.load(Ordering::Relaxed);
    if base == 0 {
        return false;
    }
    if action_playing() {
        RESETS_HELD.fetch_add(1, Ordering::Relaxed);
        return false;
    }
    let f = base + RVA_CHR_RESET_ANIM;
    let ok = readable(f, RESET_ANIM_BYTES.len())
        && core::slice::from_raw_parts(f as *const u8, RESET_ANIM_BYTES.len()) == RESET_ANIM_BYTES;
    if !ok {
        ANIM_RESET_REFUSED.fetch_add(1, Ordering::Relaxed);
        return false;
    }
    // Same guard the game's own path relies on: the animation module exists.
    let has_anim = read_ptr(chr + 0x1ff8).and_then(|sm| read_ptr(sm + 0x28)).is_some();
    if !has_anim {
        return false;
    }
    let reset: ChrResetAnimFn = core::mem::transmute(f);
    reset(chr);
    ANIM_RESETS.fetch_add(1, Ordering::Relaxed);
    true
}

/// Call the native ForcePlayAnimation once its first bytes match 1.06.
unsafe fn force_anim(chr: usize, anim_id: u32) -> bool {
    let base = MODULE_BASE.load(Ordering::Relaxed);
    if base == 0 {
        return false;
    }
    let f = base + RVA_FORCE_PLAY_ANIM;
    let ok = readable(f, FORCE_PLAY_ANIM_BYTES.len())
        && core::slice::from_raw_parts(f as *const u8, FORCE_PLAY_ANIM_BYTES.len()) == FORCE_PLAY_ANIM_BYTES;
    if !ok {
        FORCE_REFUSED.fetch_add(1, Ordering::Relaxed);
        return false;
    }
    let Some(module) = read_ptr(chr + 0x1ff8).and_then(|sm| read_ptr(sm + 0x58)) else {
        FORCE_REFUSED.fetch_add(1, Ordering::Relaxed);
        return false;
    };
    if !readable(module, 0x18) {
        FORCE_REFUSED.fetch_add(1, Ordering::Relaxed);
        return false;
    }
    let play: ForcePlayAnimFn = core::mem::transmute(f);
    play(module, anim_id);
    FORCES.fetch_add(1, Ordering::Relaxed);
    true
}

/// True for 1.5 s after a forced action, so stop/periodic resets do not cut it.
fn action_playing() -> bool {
    LAST_FORCE_AT
        .lock()
        .map(|t| t.elapsed() < std::time::Duration::from_millis(1500))
        .unwrap_or(false)
}

fn remote_anim() -> Option<u32> {
    let fresh = REMOTE_SEEN_AT
        .lock()
        .map(|t| t.elapsed() < std::time::Duration::from_secs(1))
        .unwrap_or(false);
    if !fresh {
        return None;
    }
    let m = crate::global()?;
    let snap = m.remote_player.lock().clone()?;
    Some(snap.animation_id)
}

/// Remote animation changes: log them, learn the id it shows at rest, and
/// when it starts something else while standing, play the action animation.
/// Remote action → puppet animation, by id family (run 23 catalog on c1020:
/// 3000-3010 are its attacks; the owner saw 3002 as a jump, 3003/3004 as a
/// dodge-like lunge). Block and parry have no counterpart found yet.
/// Puppet with the player's own animation set (c0000): play the remote
/// player's action ids as they are. `SEKIRO_COOP_PUPPET_ANIM_PASSTHROUGH=1`.
static PASSTHROUGH: Lazy<bool> = Lazy::new(|| {
    std::env::var("SEKIRO_COOP_PUPPET_ANIM_PASSTHROUGH").map(|v| v == "1").unwrap_or(false)
});

fn map_action(id: u32) -> Option<u32> {
    if *PASSTHROUGH {
        // Animation-like ids only (6+ digits); 401, 432, 600, 11, 1151 ... are
        // movement/turn states (run 24), left to the position path.
        return Some(id).filter(|v| (100_000..=99_999_999).contains(v));
    }
    if let Some((_, to)) = ACTION_MAP.iter().find(|(from, _)| *from == id) {
        return Some(*to).filter(|t| *t != 0);
    }
    match id {
        // Ground combo: 503000x0 = hit x+1. 503001x0 follows each hit and
        // 50308050 follows the jump attack (run 23 order): not new actions.
        50300000..=50300090 if id % 10 == 0 => Some(match (id % 100) / 10 {
            0 => 3000,
            1 => 3001,
            2 => 3005,
            _ => 3006,
        }),
        // Owl (c5060) puppet, run 28 slow catalog (owner): 3093 = block,
        // 3063 = dodge (not sure). Parry: no own lookalike found, block used.
        // Jump: none picked (too many jump attacks); the position path lifts it.
        50203000 => Some(3093),         // block (start; 50203010/12 follow)
        50002000 => Some(3093),         // parry (guess: run 23 order)
        // Dodge: 3063 was a two-hit combo (run 29); of the candidates the
        // owner picked 3064, a backstep that throws firecrackers (run 30).
        213300 | 213301 => Some(3064),
        _ => None,
    }
}
static REMOTE_ACTION_SEQ: Mutex<Option<u32>> = Mutex::new(None);
static ACTION_LOGGED: AtomicU64 = AtomicU64::new(0);

unsafe fn remote_action_step(chr: usize) {
    let fresh = REMOTE_SEEN_AT
        .lock()
        .map(|t| t.elapsed() < std::time::Duration::from_secs(1))
        .unwrap_or(false);
    if !fresh {
        return;
    }
    let Some(m) = crate::global() else { return };
    let Some(snap) = m.remote_player.lock().clone() else { return };
    let mut last = REMOTE_ACTION_SEQ.lock();
    let first = last.is_none();
    if *last == Some(snap.action_seq) {
        return;
    }
    *last = Some(snap.action_seq);
    drop(last);
    if first {
        return;
    }
    let to = map_action(snap.action_id);
    let forced = match to {
        Some(to) => force_anim(chr, to),
        None => false,
    };
    let now = std::time::Instant::now();
    if forced {
        *LAST_FORCE_AT.lock() = Some(now);
        *FORCE_CHECK.lock() = Some((now, to.unwrap_or(0)));
    }
    if ACTION_LOGGED.fetch_add(1, Ordering::Relaxed) < 500 {
        tracing::info!(
            t_ms = T0.elapsed().as_millis() as u64,
            action = snap.action_id,
            seq = snap.action_seq,
            puppet = ?to,
            forced,
            "puppet remote action"
        );
    }
}

unsafe fn action_step(chr: usize, moving: bool) {
    let Some(a) = remote_anim() else { return };
    let now = std::time::Instant::now();
    let prev = *REMOTE_ANIM.lock();
    match prev {
        Some((cur, since)) if cur == a => {
            // Unchanged: after 1 s standing still this is the rest animation.
            if !moving && now.duration_since(since) >= std::time::Duration::from_secs(1) {
                *REST_ANIM.lock() = Some(a);
            }
            catalog_step(chr, moving, now.duration_since(since));
        }
        _ => {
            *REMOTE_ANIM.lock() = Some((a, now));
            // Run 21: the remote's own id says which action it is. Play the
            // mapped puppet animation (attacks while moving too).
            // Run 23: actions come from the action slots now (remote_action_step);
            // the base layer is only logged.
            let mapped: Option<u32> = None;
            let forced = false;
            if forced {
                *LAST_FORCE_AT.lock() = Some(now);
                *FORCE_CHECK.lock() = Some((now, mapped.unwrap_or(0)));
            }
            let mut ev = ANIM_EVENTS.lock();
            if ev.len() < 64 {
                ev.push((now.duration_since(*T0).as_millis(), prev.map(|p| p.0).unwrap_or(0), a, moving, forced));
            }
        }
    }
}

/// Remote animation id → puppet animation id (0 = play nothing).
/// Default from run 21 (owner's test order, derived): 790040 = attack,
/// 50203000 = block, 201001 = jump (the position already lifts the puppet),
/// 213300/213301 = dodge. Override: `SEKIRO_COOP_PUPPET_MAP=790040:3000,...`.
static ACTION_MAP: Lazy<Vec<(u32, u32)>> = Lazy::new(|| {
    let spec = std::env::var("SEKIRO_COOP_PUPPET_MAP")
        .unwrap_or_default();
    spec.split(',')
        .filter_map(|kv| {
            let (k, v) = kv.split_once(':')?;
            Some((k.trim().parse().ok()?, v.trim().parse().ok()?))
        })
        .collect()
});

/// Catalog: while the remote stands still, play candidate puppet animations
/// one by one so the owner can say which looks like a block, a dodge, a jump.
/// `SEKIRO_COOP_PUPPET_CATALOG=id,id,...` (empty = off).
static CATALOG: Lazy<Vec<u32>> = Lazy::new(|| {
    std::env::var("SEKIRO_COOP_PUPPET_CATALOG")
        .unwrap_or_default()
        .split(',')
        .filter_map(|v| v.trim().parse().ok())
        .collect()
});
static CATALOG_MS: Lazy<u64> = Lazy::new(|| {
    std::env::var("SEKIRO_COOP_PUPPET_CATALOG_MS").ok().and_then(|v| v.parse().ok()).unwrap_or(3000)
});
static CATALOG_OK: Mutex<Vec<u32>> = Mutex::new(Vec::new());
static CATALOG_NEXT: AtomicUsize = AtomicUsize::new(0);
static CATALOG_LAST: Mutex<Option<std::time::Instant>> = Mutex::new(None);

/// FPS test (run 26: 25-30 FPS with the spawned Owl). `SEKIRO_COOP_PUPPET_PHASES=1`
/// runs, counted from the pin: A 0-40 s normal; B 40-80 s warps only (no raw
/// position writes); C 80-120 s no position writes at all; D after 120 s normal,
/// and the catalog (if set) only in D.
static PHASES: Lazy<bool> = Lazy::new(|| {
    std::env::var("SEKIRO_COOP_PUPPET_PHASES").map(|v| v == "1").unwrap_or(false)
});
static PIN_TIME: Mutex<Option<std::time::Instant>> = Mutex::new(None);
static PHASE_LOGGED: AtomicUsize = AtomicUsize::new(usize::MAX);

fn phase() -> char {
    if !*PHASES {
        return 'D';
    }
    let Some(t) = *PIN_TIME.lock() else { return 'A' };
    let s = t.elapsed().as_secs();
    let p = match s {
        0..=39 => 'A',
        40..=79 => 'B',
        80..=119 => 'C',
        _ => 'D',
    };
    let idx = p as usize;
    if PHASE_LOGGED.swap(idx, Ordering::Relaxed) != idx {
        tracing::info!(phase = %p, "puppet FPS test phase");
    }
    p
}

unsafe fn catalog_step(chr: usize, moving: bool, still_for: std::time::Duration) {
    if CATALOG.is_empty() || moving || still_for < std::time::Duration::from_secs(3) || phase() != 'D' {
        return;
    }
    if T0.elapsed() < std::time::Duration::from_secs(15) {
        return;
    }
    let due = CATALOG_LAST
        .lock()
        .map(|t| t.elapsed() >= std::time::Duration::from_millis(*CATALOG_MS))
        .unwrap_or(true);
    if !due {
        return;
    }
    let i = CATALOG_NEXT.load(Ordering::Relaxed);
    let Some(&id) = CATALOG.get(i) else { return };
    let now = std::time::Instant::now();
    *CATALOG_LAST.lock() = Some(now);
    CATALOG_NEXT.store(i + 1, Ordering::Relaxed);
    if force_anim(chr, id) {
        *LAST_FORCE_AT.lock() = Some(now);
        *FORCE_CHECK.lock() = Some((now, id));
        tracing::info!(number = i + 1, of = CATALOG.len(), id, "puppet catalog play");
    }
}

/// Physics module +0x70 is NOT a quaternion: for the Hero it reads
/// `[0, yaw, 0, 0]` with yaw in radians (run 14). Calibrated against the
/// Hero's movement direction (dx, dz) over 8 samples: yaw = atan2(-dx, -dz).
fn yaw_rot(yaw: f32) -> [f32; 4] {
    [0.0, yaw, 0.0, 0.0]
}

fn wrap_pi(a: f32) -> f32 {
    let tau = std::f32::consts::TAU;
    let mut x = (a + std::f32::consts::PI) % tau;
    if x < 0.0 {
        x += tau;
    }
    x - std::f32::consts::PI
}

/// Hero rotation quaternion (physics module +0x70), for calibration logs.
fn hero_rot() -> Option<[f32; 4]> {
    let m = crate::global()?;
    let addrs = m.base_addrs.as_ref()?;
    let base = MODULE_BASE.load(Ordering::Relaxed);
    let root = base + addrs.player_position;
    unsafe {
        let a = read_ptr(root)?;
        let chr = read_ptr(a + 0x88)?;
        let sm = read_ptr(chr + 0x1ff8)?;
        let phys = read_ptr(sm + 0x68)?;
        read_checked::<[f32; 4]>(phys + 0x70)
    }
}

/// The Hero's facing in radians (physics +0x70, element 1), for the snapshot.
pub fn hero_yaw() -> Option<f32> {
    hero_rot().map(|r| r[1]).filter(|y| y.is_finite())
}

/// Puppet rotation: the physics module keeps `[0, yaw, 0, 0]` at +0x70.
unsafe fn rot_of(chr: usize) -> Option<[f32; 4]> {
    let sm = read_ptr(chr + 0x1ff8)?;
    let phys = read_ptr(sm + 0x68)?;
    read_checked::<[f32; 4]>(phys + 0x70)
}

/// EnableLogic(false) for this one character: clear bit 0 of ChrCtrl+0x190.
unsafe fn logic_off(chr: usize) {
    if let Some(ctrl) = read_ptr(chr + 0x50) {
        if let Some(b) = read_checked::<u8>(ctrl + 0x190) {
            if b & 1 != 0 {
                RawPtr(ctrl + 0x190).write::<u8>(b & 0xfe);
                LOGIC_WRITES.fetch_add(1, Ordering::Relaxed);
            }
        }
    }
}

// ---------------------------------------------------------------- writer

/// Hold the pinned target beside the Hero. Called from the SetFlag detour
/// (game thread) or from the ticker, depending on `SEKIRO_COOP_PUPPET_WRITER`.
fn write_step(from_ticker: bool) {
    if !*ENABLED || from_ticker != *WRITER_TICKER {
        return;
    }
    // The native warp may itself call SetFlag (our detour). Never re-enter.
    if IN_STEP.swap(true, Ordering::Acquire) {
        return;
    }
    struct Clear;
    impl Drop for Clear {
        fn drop(&mut self) {
            IN_STEP.store(false, Ordering::Release);
        }
    }
    let _clear = Clear;
    let Some(t) = *TARGET.lock() else { return };
    if CUR_FRAME.load(Ordering::Relaxed) < PAUSE_UNTIL.load(Ordering::Relaxed) {
        return;
    }
    // At most one write per ~8 ms: SetFlag fires several times per frame,
    // and writing every call cost frame time without moving the puppet more.
    {
        let mut last = LAST_WRITE_AT.lock();
        let now = std::time::Instant::now();
        if let Some(prev) = *last {
            if now.duration_since(prev) < std::time::Duration::from_millis(8) {
                return;
            }
        }
        *last = Some(now);
    }
    let Some(hero) = hero_pos() else { return };
    unsafe {
        if read_checked::<u32>(t.ptr + 0x08) != Some(t.handle) {
            STALE.fetch_add(1, Ordering::Relaxed);
            *LAST_HANDLE.lock() = Some((t.handle, t.char_id));
            *TARGET.lock() = None;
            *LAST_WRITTEN.lock() = None;
            return;
        }
        // Make the puppet harmless first, whatever happens below (run 11: with
        // no network data the step returned before this and it attacked).
        if *LOGIC_OFF {
            logic_off(t.ptr);
        }
        if let Some(team) = *TEAM {
            if let Some(cur_team) = read_checked::<u8>(t.ptr + 0x74) {
                if cur_team != team {
                    TEAM_BEFORE.lock().get_or_insert(cur_team);
                    RawPtr(t.ptr + 0x74).write::<u8>(team);
                    TEAM_WRITES.fetch_add(1, Ordering::Relaxed);
                }
            }
        }
        if *RESET_ANIM {
            let due = LAST_RESET_AT
                .lock()
                .map(|t| t.elapsed() >= std::time::Duration::from_secs(2))
                .unwrap_or(true);
            if due && reset_anim(t.ptr) {
                *LAST_RESET_AT.lock() = Some(std::time::Instant::now());
            }
        }
        let ph = phase();
        if ph == 'C' {
            return;
        }
        // Input copy (run 35): the PlayerIns puppet ignores ForcePlayAnimation;
        // player moves come from the manipulator. Copy the Hero's manipulator
        // state (ChrManipulator base fields +0x10..+0x300, laid out by its ctor
        // 0x7e3220: vectors, two 81-byte action arrays, 81 dwords) into the
        // puppet's manipulator (ChrIns+0x58) on every step.
        if *INPUT_COPY {
            input_copy(t.ptr);
        }
        // Run 56: the clone is updated rarely. Script command
        // ChrCmd_ForceSetOmissionLevel (EventChrEntity vt+0x1e0 = 0x74fae0) sets
        // bit 0 of ChrIns+0x1a10 (force) and ChrIns+0x110 (omission level, when
        // ChrIns+0x1ff0 == -2). Force the clone to the Hero's level every step.
        if *CLONE_NOOMIT {
            if let Some(hero_c) = hero_chr() {
                let hl = read_checked::<u32>(hero_c + 0x110);
                let cl = read_checked::<u32>(t.ptr + 0x110);
                let cf = read_checked::<u8>(t.ptr + 0x1a10);
                if !NOOMIT_LOGGED.swap(true, Ordering::Relaxed) {
                    tracing::info!(hero_level = ?hl, clone_level = ?cl, clone_flags = ?cf,
                        hero_1ff0 = ?read_checked::<i32>(hero_c + 0x1ff0), clone_1ff0 = ?read_checked::<i32>(t.ptr + 0x1ff0),
                        hero_flags = ?read_checked::<u8>(hero_c + 0x1a10), "omission: before");
                }
                if let (Some(hl), Some(cf)) = (hl, cf) {
                    RawPtr(t.ptr + 0x110).write::<u32>(hl);
                    RawPtr(t.ptr + 0x1a10).write::<u8>(cf | 1);
                }
            }
        }
        if *CORRECT_ONLY {
            // Run 39: the clone is created at the world origin and, far from
            // the Hero, its manipulator was never updated. Put it beside the
            // Hero once, whatever the network says.
            {
                let mut lw = LAST_WRITTEN.lock();
                if lw.is_none() {
                    let pos = [hero[0] + 2.0, hero[1], hero[2]];
                    if let Some(r) = rot_of(t.ptr) {
                        if native_warp(t.ptr, pos, r) {
                            *lw = Some(pos);
                        }
                    }
                }
            }
            // Input drives the clone; only pull it back when it drifts.
            if let (Some(r), Some(p)) = (remote_target(), phys_pos_addr(t.ptr)) {
                let cur: [f32; 3] = RawPtr(p).read();
                let off_by = dist([cur[0], 0.0, cur[2]], [r[0], 0.0, r[2]]);
                let due = LAST_WARP_AT
                    .lock()
                    .map(|w| w.elapsed() >= std::time::Duration::from_millis(250))
                    .unwrap_or(true);
                // Run 41: warps every ~1.3 s read as teleports. Nudge the
                // position a little each step instead, warp only when far off.
                if off_by > 0.3 && off_by <= *CORRECT_M && (cur[1] - r[1]).abs() <= 3.0 {
                    let k = 0.04;
                    let np = [cur[0] + (r[0] - cur[0]) * k, cur[1], cur[2] + (r[2] - cur[2]) * k];
                    RawPtr(p).write::<[f32; 3]>(np);
                    NUDGES.fetch_add(1, Ordering::Relaxed);
                }
                if (off_by > *CORRECT_M || (cur[1] - r[1]).abs() > 3.0) && due {
                    if let Some(rot) = rot_of(t.ptr) {
                        if native_warp(t.ptr, r, rot) {
                            *LAST_WARP_AT.lock() = Some(std::time::Instant::now());
                            CORRECTIONS.fetch_add(1, Ordering::Relaxed);
                        }
                    }
                }
                REMOTE_FRESH_STEPS.fetch_add(1, Ordering::Relaxed);
            }
            return;
        }
        {
            if *NO_POS {
                // Put it beside the Hero once, then leave movement to the input.
                let mut lw = LAST_WRITTEN.lock();
                if lw.is_none() {
                    let pos = [hero[0] + 2.0, hero[1], hero[2]];
                    if let Some(r) = rot_of(t.ptr) {
                        if native_warp(t.ptr, pos, r) {
                            *lw = Some(pos);
                        }
                    }
                }
                return;
            }
        }
        let Some(p) = phys_pos_addr(t.ptr) else {
            STALE.fetch_add(1, Ordering::Relaxed);
            return;
        };
        let cur: [f32; 3] = RawPtr(p).read();
        {
            let mut last = LAST_WRITTEN.lock();
            if let Some(l) = *last {
                if dist([cur[0], 0.0, cur[2]], [l[0], 0.0, l[2]]) > 0.01 {
                    OVERWRITTEN.fetch_add(1, Ordering::Relaxed);
                    *LAST_SEEN.lock() = Some(cur);
                }
            }
            let h = heading_update(hero);
            // Target on the ground plane, and the height we expect it at.
            let (tx, ty, tz) = if *SOURCE_NET {
                match remote_target() {
                    Some(r) => {
                        REMOTE_FRESH_STEPS.fetch_add(1, Ordering::Relaxed);
                        (r[0], r[1], r[2])
                    }
                    None => {
                        // No network position: hold still where it was last
                        // put (or where it stands now), instead of letting its
                        // last animation carry it away (run 12).
                        REMOTE_STALE_STEPS.fetch_add(1, Ordering::Relaxed);
                        // Data stopped: treat it like a stop, so the puppet does
                        // not stand in its run cycle (run 19).
                        if TARGET_MOVING.swap(false, Ordering::Relaxed) && *RESET_ANIM {
                            if reset_anim(t.ptr) {
                                STOP_RESETS.fetch_add(1, Ordering::Relaxed);
                                *LAST_RESET_AT.lock() = Some(std::time::Instant::now());
                            }
                        }
                        let hold = last.unwrap_or(cur);
                        if last.is_none() {
                            *last = Some(hold);
                        }
                        RawPtr(p).write::<[f32; 3]>(hold);
                        return;
                    }
                }
            } else {
                (hero[0] + h[0] * *OFFSET_M, hero[1], hero[2] + h[1] * *OFFSET_M)
            };
            // Keep the height the game resolved (cur[1]): forcing the Hero's
            // height put the puppet into slopes and made physics push it out
            // every frame (FPS fell to ~15 in the first runs).
            // ...but moving it sideways without support let it fall through
            // the world (run 4: y went -43 → -265, then despawn). So keep the
            // game's height only while it stays near the Hero's; otherwise put
            // it 0.2 m above the Hero's height and let it land.
            let y = if cur[1] < ty - 1.0 || cur[1] > ty + 2.0 {
                RESCUED.fetch_add(1, Ordering::Relaxed);
                ty + 0.2
            } else {
                cur[1]
            };
            // When the target stops, reset the puppet's animation at once so
            // it does not keep its run cycle until the next 2 s reset (run 17).
            {
                let now = std::time::Instant::now();
                let mut mt = MOVE_TRACK.lock();
                let moved = match *mt {
                    Some((p0, _)) => {
                        let (dx, dz) = (tx - p0[0], tz - p0[2]);
                        dx * dx + dz * dz > 0.05 * 0.05
                    }
                    None => true,
                };
                if moved {
                    *mt = Some(([tx, ty, tz], now));
                    TARGET_MOVING.store(true, Ordering::Relaxed);
                } else {
                    let since = mt.map(|(_, s)| s).unwrap_or(now);
                    drop(mt);
                    let still_for = now.duration_since(since);
                    if TARGET_MOVING.load(Ordering::Relaxed)
                        && still_for >= std::time::Duration::from_millis(200)
                    {
                        TARGET_MOVING.store(false, Ordering::Relaxed);
                        if *RESET_ANIM && reset_anim(t.ptr) {
                            STOP_RESETS.fetch_add(1, Ordering::Relaxed);
                            *LAST_RESET_AT.lock() = Some(now);
                        }
                    } else if !TARGET_MOVING.load(Ordering::Relaxed)
                        && still_for < std::time::Duration::from_millis(1700)
                    {
                        // Follow-up resets shortly after a stop, in case the
                        // game restarted the run cycle (run 18: rare leftovers).
                        let again = LAST_RESET_AT
                            .lock()
                            .map(|t| now.duration_since(t) >= std::time::Duration::from_millis(400))
                            .unwrap_or(true);
                        if again && *RESET_ANIM && reset_anim(t.ptr) {
                            STOP_RESETS.fetch_add(1, Ordering::Relaxed);
                            *LAST_RESET_AT.lock() = Some(now);
                        }
                    }
                }
            }
            // Yaw from the target's own movement (net) or the Hero's heading.
            let yaw = {
                let mut lt = LAST_TARGET.lock();
                let mut yg = YAW.lock();
                // Network source: the remote player's own facing wins (turns in
                // place). Otherwise derive it from the movement.
                let net_yaw = if *SOURCE_NET { remote_yaw() } else { None };
                if let Some(ny) = net_yaw {
                    YAW_TARGET.store(wrap_pi(ny + *YAW_OFFSET).to_bits(), Ordering::Relaxed);
                    HAVE_YAW_TARGET.store(true, Ordering::Relaxed);
                    *lt = Some([tx, ty, tz]);
                } else if let Some(prev) = *lt {
                    let (dx, dz) = (tx - prev[0], tz - prev[2]);
                    if dx * dx + dz * dz > 0.15 * 0.15 {
                        YAW_TARGET.store((wrap_pi((-dx).atan2(-dz) + *YAW_OFFSET)).to_bits(), Ordering::Relaxed);
                        HAVE_YAW_TARGET.store(true, Ordering::Relaxed);
                        *lt = Some([tx, ty, tz]);
                    }
                } else {
                    *lt = Some([tx, ty, tz]);
                }
                // Turn toward the target yaw along the shorter arc.
                if HAVE_YAW_TARGET.load(Ordering::Relaxed) {
                    let goal = f32::from_bits(YAW_TARGET.load(Ordering::Relaxed));
                    let now = yg.unwrap_or(goal);
                    *yg = Some(wrap_pi(now + wrap_pi(goal - now) * 0.25));
                }
                *yg
            };
            // Ease toward the network target instead of jumping to each packet.
            let (sx, sz) = if *SOURCE_NET {
                let from = last.unwrap_or(cur);
                let k = SMOOTH.clamp(0.05, 1.0);
                (from[0] + (tx - from[0]) * k, from[2] + (tz - from[2]) * k)
            } else {
                (tx, tz)
            };
            let desired = [sx, y, sz];
            if let Some(yaw) = yaw {
                if let Some(rp) = read_ptr(t.ptr + 0x1ff8).and_then(|sm| read_ptr(sm + 0x68)) {
                    if readable(rp + 0x70, 16) {
                        RawPtr(rp + 0x70).write::<[f32; 4]>(yaw_rot(yaw));
                    }
                }
            }
            let off_by = dist([cur[0], 0.0, cur[2]], [tx, 0.0, tz]);
            let warp_due = {
                let last_warp = *LAST_WARP_AT.lock();
                match last_warp {
                    None => true,
                    Some(w) => w.elapsed() >= std::time::Duration::from_millis(*WARP_MS),
                }
            };
            // While the target stands still, only warp to fix a big error: a
            // warp reads as movement and can restart the run cycle.
            let moving = TARGET_MOVING.load(Ordering::Relaxed);
            if *SOURCE_NET {
                action_step(t.ptr, moving);
                remote_action_step(t.ptr);
            }
            let warped = !*MODE_RAW
                && ((warp_due && (moving || ph == 'B')) || off_by > 2.0)
                && (*YAW.lock())
                    .map(yaw_rot)
                    .or_else(|| rot_of(t.ptr))
                    .map(|r| native_warp(t.ptr, desired, r))
                    .unwrap_or(false);
            if warped {
                *LAST_WARP_AT.lock() = Some(std::time::Instant::now());
            } else if ph != 'B' {
                RawPtr(p).write::<[f32; 3]>(desired);
                RAW_STEPS.fetch_add(1, Ordering::Relaxed);
            }
            for (i, off) in MIRRORS.iter().enumerate() {
                let a = t.ptr + off;
                if !readable(a, 12) {
                    continue;
                }
                if let Some(l) = *last {
                    let m: [f32; 3] = RawPtr(a).read();
                    if dist([m[0], 0.0, m[2]], [l[0], 0.0, l[2]]) > 0.01 {
                        MIRROR_OVERWRITTEN[i].fetch_add(1, Ordering::Relaxed);
                    }
                }
                RawPtr(a).write::<[f32; 3]>(desired);
            }
            *last = Some(desired);
        }
        WRITES.fetch_add(1, Ordering::Relaxed);
        let _ = WRITER_TID.set(format!("{:?}", std::thread::current().id()));
    }
}

/// Hook point for the SetFlag detour.
pub fn on_game_thread() {
    if *ENABLED {
        spawn_step();
        write_step(false);
    }
}

// ---------------------------------------------------------------- spawn

/// Debug character creator (1.06, offline): `WorldChrMan` (global RVA
/// 0x3d7a1e0) +0x3148 holds a `CSDebugChrCreator` (vftable 0x2a952a8), built by
/// the WorldChrMan constructor and stepped by its update. Fields, as
/// `CSDebugChrPerfChecker::_STEP_CreateChr` (0xc15f00) fills them:
/// +0x118 NpcParam id, +0x114 ThinkParam id, +0x1a4 distance in front of the
/// Hero, +0x1a8 height offset, +0x1ac angle. `0xc0f2a0(creator, model)` stores
/// the model number at +0x1a0 when the creator is idle (+0x1b0 = 1) and
/// starts the create (0xc10880: position from the Hero, name "c%04d", then
/// load → create).
const RVA_WORLD_CHR_MAN: usize = 0x3d7a1e0;
const OFF_CHR_CREATOR: usize = 0x3148;
const RVA_CHR_CREATOR_VT: usize = 0x2a952a8;
const RVA_CHR_CREATOR_REQUEST: usize = 0xc0f2a0;
const CHR_CREATOR_REQUEST_BYTES: [u8; 15] =
    [0x80, 0xb9, 0xb0, 0x01, 0x00, 0x00, 0x00, 0x74, 0x0b, 0x89, 0x91, 0xa0, 0x01, 0x00, 0x00];
type ChrCreatorRequestFn = unsafe extern "system" fn(creator: usize, model: u32);

/// `SEKIRO_COOP_SPAWN=<model>[:<npc param>|auto[:<think param>]]`, e.g. `5060:auto`
/// (Owl). `auto` takes the first NpcParam row in model*10000 .. +9999.
static SPAWN: Lazy<Option<(u32, Option<u32>, Option<u32>)>> = Lazy::new(|| {
    let v = std::env::var("SEKIRO_COOP_SPAWN").ok()?;
    let mut it = v.split(':');
    let model: u32 = it.next()?.trim().parse().ok()?;
    let npc = match it.next().map(str::trim) {
        None | Some("auto") | Some("") => None,
        Some(n) => n.parse().ok(),
    };
    // Think param defaults to the NpcParam id (the usual pairing); the
    // puppet's logic is switched off once it is pinned anyway.
    let think = it.next().and_then(|t| t.trim().parse().ok());
    Some((model, npc, think))
});
static SPAWN_DONE: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
static SPAWN_FIRST_SEEN: Mutex<Option<std::time::Instant>> = Mutex::new(None);
pub static SPAWNED_NPC: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);
/// (creator, value of creator+0x1b8 before the request, requested at).
/// STEP_LoadWait ends with `creator+0x1b8 = 0xa492e0(WorldChrMan, &creator+0x100)`:
/// the created ChrIns (run 25: the Owl spawned but never reached the
/// ApplyEffect-fed registry, so it was never pinned and killed the owner).
static SPAWN_WAIT: Mutex<Option<(usize, usize, std::time::Instant)>> = Mutex::new(None);

fn npc_rows_for(model: u32) -> Vec<u32> {
    let base = MODULE_BASE.load(Ordering::Relaxed);
    let lo = model * 10_000;
    let hi = lo + 9_999;
    unsafe {
        sekiro_sdk_sys::paramrepo::open_param(
            sekiro_sdk_sys::paramrepo::SOLO_PARAM_REPOSITORY_RVA_V1_06,
            base,
            sekiro_sdk_sys::paramrepo::ParamKind::NpcParam,
        )
        .map(|t| t.rows().map(|(id, _)| id).filter(|id| (lo..=hi).contains(id)).collect())
        .unwrap_or_default()
    }
}

static INPUT_COPY: Lazy<bool> = Lazy::new(|| {
    std::env::var("SEKIRO_COOP_PUPPET_INPUT_COPY").map(|v| v == "1").unwrap_or(false)
});
static NO_POS: Lazy<bool> = Lazy::new(|| {
    std::env::var("SEKIRO_COOP_PUPPET_NOPOS").map(|v| v == "1").unwrap_or(false)
});
static INPUT_COPIES: AtomicU64 = AtomicU64::new(0);
static CORRECT_ONLY: Lazy<bool> = Lazy::new(|| {
    std::env::var("SEKIRO_COOP_PUPPET_CORRECT_ONLY").map(|v| v == "1").unwrap_or(false)
});
static CORRECT_M: Lazy<f32> = Lazy::new(|| {
    std::env::var("SEKIRO_COOP_PUPPET_CORRECT_M").ok().and_then(|v| v.parse().ok()).unwrap_or(1.5)
});
static CORRECTIONS: AtomicU64 = AtomicU64::new(0);
static CLONE_NOOMIT: Lazy<bool> = Lazy::new(|| {
    std::env::var("SEKIRO_COOP_CLONE_NOOMIT").map(|v| v == "1").unwrap_or(false)
});
static NOOMIT_LOGGED: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
static NUDGES: AtomicU64 = AtomicU64::new(0);
static INPUT_LOGGED: AtomicU64 = AtomicU64::new(0);
const MANIP_COPY_FROM: usize = 0x10;
const MANIP_COPY_TO: usize = 0x300;

unsafe fn input_copy(puppet: usize) {
    let Some(hero) = hero_chr() else { return };
    if hero == puppet {
        return;
    }
    let (Some(src), Some(dst)) = (read_ptr(hero + 0x58), read_ptr(puppet + 0x58)) else { return };
    if src == dst || !readable(src, MANIP_COPY_TO) || !readable(dst, MANIP_COPY_TO) {
        return;
    }
    core::ptr::copy_nonoverlapping(
        (src + MANIP_COPY_FROM) as *const u8,
        (dst + MANIP_COPY_FROM) as *mut u8,
        MANIP_COPY_TO - MANIP_COPY_FROM,
    );
    let n = INPUT_COPIES.fetch_add(1, Ordering::Relaxed);
    if n % 500 == 0 && INPUT_LOGGED.fetch_add(1, Ordering::Relaxed) < 40 {
        let base = MODULE_BASE.load(Ordering::Relaxed);
        let v: [f32; 4] = RawPtr(src + 0x10).read();
        let flags: [u8; 16] = RawPtr(src + 0xf4).read();
        tracing::info!(
            copies = n + 1,
            src_vt = ?read_checked::<usize>(src).map(|x| format!("{:#x}", x.wrapping_sub(base))),
            dst_vt = ?read_checked::<usize>(dst).map(|x| format!("{:#x}", x.wrapping_sub(base))),
            v10 = ?v,
            flags = ?flags,
            puppet_anim = ?anim_of(puppet),
            dst = format!("{dst:#x}"),
            puppet_ctrl_120 = ?read_ptr(puppet + 0x50).and_then(|c| read_ptr(c + 0x120)).map(|v| format!("{v:#x}")),
            puppet_ctrl_358 = ?read_ptr(puppet + 0x50).and_then(|c| read_ptr(c + 0x358)).map(|v| format!("{v:#x}")),
            src = format!("{src:#x}"),
            hero_ctrl_120 = ?read_ptr(hero + 0x50).and_then(|c| read_ptr(c + 0x120)).map(|v| format!("{v:#x}")),
            hero_ctrl_358 = ?read_ptr(hero + 0x50).and_then(|c| read_ptr(c + 0x358)).map(|v| format!("{v:#x}")),
            puppet_type = ?read_checked::<u32>(puppet + 0x70),
            "puppet input copy"
        );
    }
}

// ---------------------------------------------------------------- clone pad

/// Run 37: a PlayerIns with its own PadManipulator (kind 1) took all of the
/// owner's input (the Hero stopped responding). PadManipulator vt[0]
/// (0x7f2210 → 0x7ef930) is its per-frame update that reads the pad. For the
/// clone only, swap in a copy of its vtable whose slot 0 does not read the pad
/// but copies the Hero manipulator's processed state (the source the network
/// will replace later). Ranges skip the sub-object pointers at +0x350/+0x358
/// and the qwords at +0x300, +0x410/+0x418, +0x428/+0x430.
static CLONE_PAD: Lazy<bool> = Lazy::new(|| {
    std::env::var("SEKIRO_COOP_CLONE_PAD").map(|v| v == "1").unwrap_or(false)
});
const RVA_PAD_MANIP_VT: usize = 0x29d4b30;
const PAD_COPY_RANGES: [(usize, usize); 5] =
    [(0x10, 0x300), (0x310, 0x340), (0x360, 0x410), (0x420, 0x428), (0x438, 0x450)];
static CLONE_VT: Mutex<[usize; 64]> = Mutex::new([0; 64]);
static CLONE_MANIP: AtomicUsize = AtomicUsize::new(0);
static CLONE_UPDATES: AtomicU64 = AtomicU64::new(0);

const PAD_COPY_LEN: usize = (0x300 - 0x10) + (0x340 - 0x310) + (0x410 - 0x360) + (0x428 - 0x420) + (0x450 - 0x438);
static REMOTE_MANIP: Mutex<Option<(Vec<u8>, std::time::Instant)>> = Mutex::new(None);
static REMOTE_MANIP_RX: AtomicU64 = AtomicU64::new(0);
static REMOTE_MANIP_USED: AtomicU64 = AtomicU64::new(0);

/// The Hero's manipulator state in PAD_COPY_RANGES order, for the network.
pub fn hero_manip_bytes() -> Option<Vec<u8>> {
    if !*SEND_MANIP {
        return None;
    }
    let hero = hero_chr()?;
    unsafe {
        let src = read_ptr(hero + 0x58)?;
        if read_checked::<usize>(src) != Some(MODULE_BASE.load(Ordering::Relaxed) + RVA_PAD_MANIP_VT)
            || !readable(src, 0x450)
        {
            return None;
        }
        let mut out = Vec::with_capacity(PAD_COPY_LEN);
        for (a, b) in PAD_COPY_RANGES {
            out.extend_from_slice(core::slice::from_raw_parts((src + a) as *const u8, b - a));
        }
        Some(out)
    }
}
static SEND_MANIP: Lazy<bool> = Lazy::new(|| {
    std::env::var("SEKIRO_COOP_SEND_MANIP").map(|v| v != "0").unwrap_or(true)
});

pub fn note_remote_manip(bytes: Vec<u8>) {
    if bytes.len() == PAD_COPY_LEN {
        REMOTE_MANIP_RX.fetch_add(1, Ordering::Relaxed);
        *REMOTE_MANIP.lock() = Some((bytes, std::time::Instant::now()));
    }
}

unsafe extern "system" fn clone_pad_update(this: usize, _a: usize, _b: usize, _c: usize) -> usize {
    CLONE_UPDATES.fetch_add(1, Ordering::Relaxed);
    if !readable(this, 0x450) {
        return 0;
    }
    // Network source: the remote player's input state.
    if *SOURCE_NET {
        let g = REMOTE_MANIP.lock();
        if let Some((b, at)) = g.as_ref() {
            if at.elapsed() < std::time::Duration::from_millis(500) {
                let mut off = 0;
                for (a, e) in PAD_COPY_RANGES {
                    core::ptr::copy_nonoverlapping(b.as_ptr().add(off), (this + a) as *mut u8, e - a);
                    off += e - a;
                }
                REMOTE_MANIP_USED.fetch_add(1, Ordering::Relaxed);
                return 0;
            }
        }
        // No fresh input: stand still (clear the base input state).
        drop(g);
        core::ptr::write_bytes((this + 0x10) as *mut u8, 0, 0x300 - 0x10);
        return 0;
    }
    let Some(hero) = hero_chr() else { return 0 };
    let Some(src) = read_ptr(hero + 0x58) else { return 0 };
    if src == this || !readable(src, 0x450) || !readable(this, 0x450) {
        return 0;
    }
    for (a, b) in PAD_COPY_RANGES {
        core::ptr::copy_nonoverlapping((src + a) as *const u8, (this + a) as *mut u8, b - a);
    }
    0
}

/// Run 40: slot 0 was never called. ChrCtrl's update (0x7dee05..) calls the
/// manipulator's vt[+0x48](this, dt, ChrCtrl, 0) when its logic bit is set,
/// else vt[+0x50](this, dt, ChrCtrl); PadManipulator overrides both (0x7ef120,
/// 0x7ef300). Wrap them: run the original, then overwrite the processed
/// fields with the remote (or Hero) state before ChrCtrl consumes them.
static PAD_UPD48: AtomicUsize = AtomicUsize::new(0);
static PAD_UPD50: AtomicUsize = AtomicUsize::new(0);
static CLONE_UPD_CALLS: AtomicU64 = AtomicU64::new(0);
type PadUpdFn = unsafe extern "system" fn(usize, f32, usize, usize) -> usize;

/// Which copy ranges to apply on the clone: all (default), "base" (ChrManipulator
/// part only, < 0x340), or "none" (let the clone's own pad update stand).
static CLONE_RANGES: Lazy<String> = Lazy::new(|| {
    std::env::var("SEKIRO_COOP_CLONE_RANGES").unwrap_or_else(|_| "all".into())
});

unsafe fn clone_apply_state(this: usize) {
    if CLONE_RANGES.as_str() == "none" {
        return;
    }
    let base_only = CLONE_RANGES.as_str() == "base";
    if *SOURCE_NET {
        let g = REMOTE_MANIP.lock();
        if let Some((b, at)) = g.as_ref() {
            if at.elapsed() < std::time::Duration::from_millis(500) {
                let mut off = 0;
                for (a, e) in PAD_COPY_RANGES {
                    if !(base_only && a >= 0x340) {
                        core::ptr::copy_nonoverlapping(b.as_ptr().add(off), (this + a) as *mut u8, e - a);
                    }
                    off += e - a;
                }
                REMOTE_MANIP_USED.fetch_add(1, Ordering::Relaxed);
                return;
            }
        }
        drop(g);
        core::ptr::write_bytes((this + 0x10) as *mut u8, 0, 0x300 - 0x10);
        return;
    }
    let Some(hero) = hero_chr() else { return };
    let Some(src) = read_ptr(hero + 0x58) else { return };
    if src == this || !readable(src, 0x450) {
        return;
    }
    for (a, b) in PAD_COPY_RANGES {
        if !(base_only && a >= 0x340) {
            core::ptr::copy_nonoverlapping((src + a) as *const u8, (this + a) as *mut u8, b - a);
        }
    }
    // Run 43: with only these fields the clone attacks but never walks, even
    // with no overwrite at all. Try also copying the PadManipulator
    // sub-objects (+0x350: 0x4f30 bytes, +0x358: 0x26a8 bytes), skipping every
    // qword that is a readable pointer in either object.
    if *CLONE_SUBOBJ {
        for (off, len) in [(0x350usize, 0x4f30usize), (0x358, 0x26a8)] {
            let (Some(s_obj), Some(d_obj)) = (read_ptr(src + off), read_ptr(this + off)) else { continue };
            if !readable(s_obj, len) || !readable(d_obj, len) {
                continue;
            }
            let mut i = 0;
            while i + 8 <= len {
                let sv: u64 = RawPtr(s_obj + i).read();
                let dv: u64 = RawPtr(d_obj + i).read();
                let is_ptr = |v: u64| v > 0x10000 && v % 8 == 0 && v < 0x7fff_ffff_ffff && readable(v as usize, 8);
                if !is_ptr(sv) && !is_ptr(dv) {
                    RawPtr(d_obj + i).write::<u64>(sv);
                }
                i += 8;
            }
            SUBOBJ_COPIES.fetch_add(1, Ordering::Relaxed);
        }
    }
}
static CLONE_SUBOBJ: Lazy<bool> = Lazy::new(|| {
    std::env::var("SEKIRO_COOP_CLONE_SUBOBJ").map(|v| v == "1").unwrap_or(false)
});
static SUBOBJ_COPIES: AtomicU64 = AtomicU64::new(0);

/// Run 50 scan: while the Hero ran, `[[[bag+0x28]+0x20]+0x140]+0x344` went 0 → 2.0
/// (MoveSpeedLevel-like) and stayed 0 on the clone. `[X+0x20]+0x140` looks like the
/// data pointer of an hkArray of behavior variable values (size at +0x148).
/// `SEKIRO_COOP_BVAR_COPY=1`: copy the Hero's values into the clone's array.
static BVAR_COPY: Lazy<bool> = Lazy::new(|| {
    std::env::var("SEKIRO_COOP_BVAR_COPY").map(|v| v == "1").unwrap_or(false)
});
static BVAR_COPIES: AtomicU64 = AtomicU64::new(0);
static BVAR_SIZE_LOGGED: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
static BVAR_LOGGED: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

/// Run 52: the path from the scan did not hold (`[X+0x140]` was not a pointer
/// later). Find the Havok `hkbVariableValueSet` (vftable 0x2c14748) by RTTI
/// instead: `+0x10` hkArray<word> values (data, size at +0x18), `+0x20`
/// hkArray<hkVector4>. Searched once per character from the module bag.
const RVA_HKB_VVS_VT: usize = 0x2c14748;
static VVS_CACHE: Mutex<Vec<(usize, Vec<usize>)>> = Mutex::new(Vec::new()); // (chr, [vvs...])

unsafe fn find_vvs(chr: usize) -> Vec<usize> {
    let base = MODULE_BASE.load(Ordering::Relaxed);
    let want = base + RVA_HKB_VVS_VT;
    let mut found = Vec::new();
    let Some(bag) = read_ptr(chr + 0x1ff8) else { return found };
    let mut frontier: Vec<usize> = Vec::new();
    for m in [0x10usize, 0x28, 0x58] {
        if let Some(a) = read_ptr(bag + m) {
            frontier.push(a);
        }
    }
    let mut seen = std::collections::HashSet::new();
    let mut visited = 0;
    for _depth in 0..5 {
        let mut next = Vec::new();
        for a in frontier {
            if !seen.insert(a) || !readable(a, 0x200) {
                continue;
            }
            visited += 1;
            if visited > 6000 {
                return found;
            }
            if read_checked::<usize>(a) == Some(want) {
                found.push(a);
                continue;
            }
            let mut off = 0;
            while off < 0x200 {
                let v: usize = RawPtr(a + off).read();
                if v > 0x10000 && v % 8 == 0 && v < 0x7fff_ffff_ffff && v != chr {
                    next.push(v);
                }
                off += 8;
            }
        }
        frontier = next;
    }
    found
}

unsafe fn vvs_of(chr: usize) -> Option<usize> {
    let base = MODULE_BASE.load(Ordering::Relaxed);
    {
        let c = VVS_CACHE.lock();
        if let Some((_, v)) = c.iter().find(|(k, _)| *k == chr) {
            return v.first().copied().filter(|a| read_checked::<usize>(*a) == Some(base + RVA_HKB_VVS_VT));
        }
    }
    let v = find_vvs(chr);
    tracing::info!(chr = format!("{chr:#x}"), found = ?v.iter().map(|a| format!("{a:#x}")).collect::<Vec<_>>(),
        sizes = ?v.iter().map(|a| (read_checked::<i32>(a + 0x18), read_checked::<i32>(a + 0x28))).collect::<Vec<_>>(),
        "bvar copy: hkbVariableValueSet search");
    VVS_CACHE.lock().push((chr, v.clone()));
    v.first().copied()
}

unsafe fn bvar_copy_from_hero(clone_chr: usize) {
    if !*BVAR_COPY {
        return;
    }
    let Some(hero) = hero_chr() else { return };
    let (Some(h), Some(c)) = (vvs_of(hero), vvs_of(clone_chr)) else { return };
    for (arr, elem) in [(0x10usize, 4usize), (0x20, 16)] {
        let (Some(hd), Some(cd)) = (read_ptr(h + arr), read_ptr(c + arr)) else { continue };
        let (Some(hn), Some(cn)) = (read_checked::<i32>(h + arr + 8), read_checked::<i32>(c + arr + 8)) else { continue };
        let n = hn.min(cn).max(0) as usize;
        if n == 0 || n > 4096 || hd == cd || !readable(hd, n * elem) || !readable(cd, n * elem) {
            continue;
        }
        core::ptr::copy_nonoverlapping(hd as *const u8, cd as *mut u8, n * elem);
    }
    BVAR_COPIES.fetch_add(1, Ordering::Relaxed);
}

unsafe extern "system" fn clone_upd48(this: usize, dt: f32, ctrl: usize, flag: usize) -> usize {
    CLONE_UPD_CALLS.fetch_add(1, Ordering::Relaxed);
    let orig: PadUpdFn = core::mem::transmute(PAD_UPD48.load(Ordering::Relaxed));
    let r = orig(this, dt, ctrl, flag);
    if readable(this, 0x450) {
        clone_apply_state(this);
    }
    if let Some(t) = *TARGET.lock() {
        bvar_copy_from_hero(t.ptr);
    }
    r
}

unsafe extern "system" fn clone_upd50(this: usize, dt: f32, ctrl: usize, flag: usize) -> usize {
    CLONE_UPD_CALLS.fetch_add(1, Ordering::Relaxed);
    let orig: PadUpdFn = core::mem::transmute(PAD_UPD50.load(Ordering::Relaxed));
    let r = orig(this, dt, ctrl, flag);
    if readable(this, 0x450) {
        clone_apply_state(this);
    }
    if let Some(t) = *TARGET.lock() {
        bvar_copy_from_hero(t.ptr);
    }
    r
}

/// Swap the clone's manipulator vtable once (game thread).
unsafe fn clone_pad_install(chr: usize) {
    if !*CLONE_PAD || CLONE_MANIP.load(Ordering::Relaxed) != 0 {
        return;
    }
    let base = MODULE_BASE.load(Ordering::Relaxed);
    let Some(m) = read_ptr(chr + 0x58) else { return };
    if read_checked::<usize>(m) != Some(base + RVA_PAD_MANIP_VT) || !readable(m, 0x450) {
        tracing::warn!(vt = ?read_checked::<usize>(m).map(|v| format!("{:#x}", v.wrapping_sub(base))), "clone pad: not a PadManipulator, not swapping");
        CLONE_MANIP.store(usize::MAX, Ordering::Relaxed);
        return;
    }
    let src_vt = base + RVA_PAD_MANIP_VT;
    if !readable(src_vt, 64 * 8) {
        return;
    }
    let mut vt = CLONE_VT.lock();
    for i in 0..64 {
        vt[i] = RawPtr(src_vt + i * 8).read::<usize>();
    }
    // Slot 0 looks like the pad-device getter (run 40: never reached our
    // copy). Overriding it may leave the clone without a device, which may be
    // why it stopped walking (runs 41-44). SEKIRO_COOP_CLONE_SLOT0=0 keeps it.
    if std::env::var("SEKIRO_COOP_CLONE_SLOT0").map(|v| v != "0").unwrap_or(true) {
        vt[0] = clone_pad_update as usize;
    }
    PAD_UPD48.store(vt[0x48 / 8], Ordering::Relaxed);
    PAD_UPD50.store(vt[0x50 / 8], Ordering::Relaxed);
    if vt[0x48 / 8] != base + 0x7ef120 || vt[0x50 / 8] != base + 0x7ef300 {
        tracing::warn!("clone pad: unexpected update slots, not swapping");
        CLONE_MANIP.store(usize::MAX, Ordering::Relaxed);
        return;
    }
    vt[0x48 / 8] = clone_upd48 as usize;
    vt[0x50 / 8] = clone_upd50 as usize;
    let vt_addr = vt.as_ptr() as usize;
    drop(vt);
    RawPtr(m).write::<usize>(vt_addr);
    CLONE_MANIP.store(m, Ordering::Relaxed);
    tracing::info!(manip = format!("{m:#x}"), "clone pad: vtable swapped, slot 0 copies the Hero's manipulator");
}

// ---------------------------------------------------------------- hero/clone diff

/// Run 47: the clone glides without walk animations from its first run,
/// whatever the input. Dump small-value differences between the Hero and the
/// clone (both standing) in the ChrIns, its ChrCtrl (+0x50) and the module bag
/// (+0x1ff8, first 32 module pointers), to find a state flag that gates
/// locomotion. `SEKIRO_COOP_CLONE_DIFF=1`; runs once, 15 s after the pin.
static CLONE_DIFF: Lazy<bool> = Lazy::new(|| {
    std::env::var("SEKIRO_COOP_CLONE_DIFF").map(|v| v == "1").unwrap_or(false)
});
static CLONE_DIFF_DONE: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

unsafe fn diff_region(label: &str, a: usize, b: usize, len: usize, out: &mut Vec<String>) {
    if !readable(a, len) || !readable(b, len) {
        out.push(format!("{label}: unreadable"));
        return;
    }
    let mut i = 0;
    while i + 4 <= len {
        let x: u32 = RawPtr(a + i).read();
        let y: u32 = RawPtr(b + i).read();
        if x != y && x < 0x10000 && y < 0x10000 && out.len() < 400 {
            out.push(format!("{label}+{i:#x}: hero={x:#x} clone={y:#x}"));
        }
        i += 4;
    }
}

static MODULE_LOG_DONE: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

/// Run 54: list the module bag (+0x1ff8, 40 slots) of Hero and clone by vtable RVA,
/// and the first pointers inside the behavior module (+0x28), on the ticker thread.
fn module_log_step() {
    if std::env::var("SEKIRO_COOP_MODULE_LOG").map(|v| v != "1").unwrap_or(true) || MODULE_LOG_DONE.load(Ordering::Relaxed) {
        return;
    }
    let Some(t) = *PIN_TIME.lock() else { return };
    if t.elapsed() < std::time::Duration::from_secs(5) {
        return;
    }
    MODULE_LOG_DONE.store(true, Ordering::Relaxed);
    let base = MODULE_BASE.load(Ordering::Relaxed);
    let (Some(hero), Some(tg)) = (hero_chr(), *TARGET.lock()) else { return };
    for (who, chr) in [("hero", hero), ("clone", tg.ptr)] {
        unsafe {
            let Some(bag) = read_ptr(chr + 0x1ff8) else { continue };
            let mut rows = Vec::new();
            for k in 0..40usize {
                let m = read_ptr(bag + k * 8);
                let vt = m.and_then(|m| read_checked::<usize>(m)).map(|v| v.wrapping_sub(base));
                rows.push(format!("{:#x}:{}", k * 8, vt.map(|v| format!("{v:#x}")).unwrap_or_else(|| "-".into())));
            }
            tracing::info!(who, modules = ?rows, "module bag");
            if let Some(bm) = read_ptr(bag + 0x28) {
                let mut inner = Vec::new();
                for off in (0..0x100usize).step_by(8) {
                    let p = read_ptr(bm + off);
                    let vt = p.and_then(|p| read_checked::<usize>(p)).map(|v| v.wrapping_sub(base)).filter(|v| *v < 0x4000000);
                    inner.push(format!("{off:#x}:{}", vt.map(|v| format!("{v:#x}")).unwrap_or_else(|| if p.is_some() { "ptr".into() } else { "-".into() })));
                }
                tracing::info!(who, inner = ?inner, "behavior module +0x28 fields");
            }
        }
    }
}

fn clone_diff_step() {
    module_log_step();
    if !*CLONE_DIFF || CLONE_DIFF_DONE.load(Ordering::Relaxed) {
        return;
    }
    let Some(t) = *PIN_TIME.lock() else { return };
    if t.elapsed() < std::time::Duration::from_secs(15) {
        return;
    }
    CLONE_DIFF_DONE.store(true, Ordering::Relaxed);
    let (Some(hero), Some(tg)) = (hero_chr(), *TARGET.lock()) else { return };
    let clone = tg.ptr;
    let mut out = Vec::new();
    unsafe {
        diff_region("chr", hero, clone, 0x2300, &mut out);
        if let (Some(a), Some(b)) = (read_ptr(hero + 0x50), read_ptr(clone + 0x50)) {
            diff_region("ctrl", a, b, 0x600, &mut out);
        }
        if let (Some(ba), Some(bb)) = (read_ptr(hero + 0x1ff8), read_ptr(clone + 0x1ff8)) {
            for k in 0..32usize {
                if let (Some(a), Some(b)) = (read_ptr(ba + k * 8), read_ptr(bb + k * 8)) {
                    diff_region(&format!("mod{:#x}", k * 8), a, b, 0x300, &mut out);
                }
            }
        }
    }
    for chunk in out.chunks(20) {
        tracing::info!(lines = ?chunk, "clone diff");
    }
    tracing::info!(total = out.len(), "clone diff done");
    // Run 48: copy chosen u32 fields Hero → clone once, to find the one that
    // gates walking. `SEKIRO_COOP_CLONE_PATCH=chr:0x78,chr:0x128,ctrl:0x8,...`
    if let Ok(spec) = std::env::var("SEKIRO_COOP_CLONE_PATCH") {
        for item in spec.split(',').filter(|x| !x.is_empty()) {
            let Some((region, off)) = item.split_once(':') else { continue };
            let Ok(off) = usize::from_str_radix(off.trim_start_matches("0x"), 16) else { continue };
            unsafe {
                let (a, b) = match region {
                    "chr" => (Some(hero), Some(clone)),
                    "ctrl" => (read_ptr(hero + 0x50), read_ptr(clone + 0x50)),
                    _ => (None, None),
                };
                if let (Some(a), Some(b)) = (a, b) {
                    if readable(a + off, 4) && readable(b + off, 4) {
                        let v: u32 = RawPtr(a + off).read();
                        let old: u32 = RawPtr(b + off).read();
                        RawPtr(b + off).write::<u32>(v);
                        tracing::info!(region, off = format!("{off:#x}"), old = format!("{old:#x}"), new = format!("{v:#x}"), "clone patch");
                    }
                }
            }
        }
    }
}

// ---------------------------------------------------------------- behavior var scan

/// Run 50: find where the Hero's behavior variables (MoveSpeedLevel, MoveAngle
/// ...) live. Walk pointers from the Hero's module bag (+0x1ff8: modules
/// 0x10, 0x28, 0x58) up to depth 3, read 0x400 bytes per object, and take
/// two float snapshots: A at pin+10 s (owner stands), B at pin+25 s (owner
/// runs). Report slots that are 0 in A and 0.5..3 in B, with their pointer
/// path, and the clone's value at the same path. `SEKIRO_COOP_BVAR_SCAN=1`.
static BVAR_SCAN: Lazy<bool> = Lazy::new(|| {
    std::env::var("SEKIRO_COOP_BVAR_SCAN").map(|v| v == "1").unwrap_or(false)
});
struct ScanNode {
    addr: usize,
    path: Vec<usize>, // offsets: first is the module-bag offset, then pointer offsets
}
static BVAR_NODES: Mutex<Vec<(Vec<usize>, Vec<f32>)>> = Mutex::new(Vec::new());
static BVAR_STAGE: AtomicUsize = AtomicUsize::new(0);
const BVAR_OBJ: usize = 0x400;

unsafe fn bvar_walk(chr: usize) -> Vec<ScanNode> {
    let mut out = Vec::new();
    let Some(bag) = read_ptr(chr + 0x1ff8) else { return out };
    let mut frontier: Vec<ScanNode> = Vec::new();
    for m in [0x10usize, 0x28, 0x58] {
        if let Some(a) = read_ptr(bag + m) {
            frontier.push(ScanNode { addr: a, path: vec![m] });
        }
    }
    let mut seen = std::collections::HashSet::new();
    for depth in 0..4 {
        let mut next = Vec::new();
        for n in frontier {
            if !seen.insert(n.addr) || !readable(n.addr, BVAR_OBJ) {
                continue;
            }
            if depth < 3 {
                let mut off = 0;
                while off < BVAR_OBJ {
                    let v: usize = RawPtr(n.addr + off).read();
                    if v > 0x10000 && v % 8 == 0 && v < 0x7fff_ffff_ffff && v != chr && !seen.contains(&v) {
                        let mut p = n.path.clone();
                        p.push(off);
                        next.push(ScanNode { addr: v, path: p });
                    }
                    off += 8;
                }
            }
            out.push(n);
            if out.len() > 4000 {
                return out;
            }
        }
        frontier = next;
    }
    out
}

unsafe fn bvar_follow(chr: usize, path: &[usize]) -> Option<usize> {
    let bag = read_ptr(chr + 0x1ff8)?;
    let mut a = read_ptr(bag + path[0])?;
    for off in &path[1..] {
        a = read_ptr(a + off)?;
    }
    Some(a)
}

fn bvar_scan_step() {
    if !*BVAR_SCAN {
        return;
    }
    let Some(t0) = *PIN_TIME.lock() else { return };
    let el = t0.elapsed().as_secs();
    let stage = BVAR_STAGE.load(Ordering::Relaxed);
    let Some(hero) = hero_chr() else { return };
    unsafe {
        if stage == 0 && el >= 10 {
            BVAR_STAGE.store(1, Ordering::Relaxed);
            let nodes = bvar_walk(hero);
            let mut store = BVAR_NODES.lock();
            store.clear();
            for n in nodes {
                if readable(n.addr, BVAR_OBJ) {
                    let f: Vec<f32> = (0..BVAR_OBJ / 4).map(|i| RawPtr(n.addr + i * 4).read::<f32>()).collect();
                    store.push((n.path, f));
                }
            }
            tracing::info!(objects = store.len(), "bvar scan: snapshot A (standing)");
        } else if stage == 1 && el >= 25 {
            BVAR_STAGE.store(2, Ordering::Relaxed);
            let clone = (*TARGET.lock()).map(|t| t.ptr);
            let store = BVAR_NODES.lock();
            let mut hits: Vec<String> = Vec::new();
            for (path, a) in store.iter() {
                let Some(addr) = bvar_follow(hero, path) else { continue };
                if !readable(addr, BVAR_OBJ) {
                    continue;
                }
                for i in 0..BVAR_OBJ / 4 {
                    let b: f32 = RawPtr(addr + i * 4).read();
                    if a[i] == 0.0 && b.is_finite() && (0.5..=3.0).contains(&b.abs()) {
                        let cv = clone
                            .and_then(|c| bvar_follow(c, path))
                            .filter(|ca| readable(*ca + i * 4, 4))
                            .map(|ca| RawPtr(ca + i * 4).read::<f32>());
                        if hits.len() < 300 {
                            hits.push(format!(
                                "{}+{:#x} hero={:.3} clone={:?}",
                                path.iter().map(|o| format!("{o:#x}")).collect::<Vec<_>>().join(">"),
                                i * 4,
                                b,
                                cv
                            ));
                        }
                    }
                }
            }
            for c in hits.chunks(20) {
                tracing::info!(lines = ?c, "bvar scan hit");
            }
            tracing::info!(total = hits.len(), "bvar scan: done (B = running)");
        }
    }
}

// ---------------------------------------------------------------- is-local hook

/// Run 55: 0x9f3430 is `IsLocalPlayer(chr)` = `[WorldChrMan]+0x88 == chr`, called
/// from ~260 places; the local-only modules (HitWall, Docking, WireAction) are
/// created behind it (0xa34c90). Guess: locomotion is also gated by it. With
/// `SEKIRO_COOP_CLONE_LOCAL=1`, hook it to answer true for the clone too.
const RVA_IS_LOCAL_PLAYER: usize = 0x9f3430;
static CLONE_CHR: AtomicUsize = AtomicUsize::new(0);
static ISLOCAL_ORIG: AtomicUsize = AtomicUsize::new(0);
static ISLOCAL_CLONE_HITS: AtomicU64 = AtomicU64::new(0);

extern "system" fn is_local_detour(chr: usize) -> u8 {
    let c = CLONE_CHR.load(Ordering::Relaxed);
    if c != 0 && chr == c {
        ISLOCAL_CLONE_HITS.fetch_add(1, Ordering::Relaxed);
        return 1;
    }
    let orig = ISLOCAL_ORIG.load(Ordering::Acquire);
    if orig == 0 {
        return 0;
    }
    let f: extern "system" fn(usize) -> u8 = unsafe { core::mem::transmute(orig) };
    f(chr)
}

fn install_is_local_hook(clone: usize) {
    if std::env::var("SEKIRO_COOP_CLONE_LOCAL").map(|v| v != "1").unwrap_or(true) {
        return;
    }
    let base = MODULE_BASE.load(Ordering::Relaxed);
    let target = base + RVA_IS_LOCAL_PLAYER;
    let expect: [u8; 11] = [0x40, 0x53, 0x48, 0x83, 0xec, 0x20, 0x48, 0x8b, 0x05, 0xa3, 0x6d];
    let ok = readable(target, expect.len())
        && unsafe { core::slice::from_raw_parts(target as *const u8, expect.len()) } == expect;
    if !ok {
        tracing::warn!("is-local hook: bytes differ, not hooking");
        return;
    }
    CLONE_CHR.store(clone, Ordering::Relaxed);
    if ISLOCAL_ORIG.load(Ordering::Relaxed) != 0 {
        return;
    }
    match sekiro_sdk_core::hook::create_hook(target, is_local_detour as usize) {
        Ok((_h, tramp)) => {
            ISLOCAL_ORIG.store(tramp as usize, Ordering::Release);
            tracing::info!(clone = format!("{clone:#x}"), "is-local hook installed: clone counts as local player");
        }
        Err(e) => tracing::warn!(err = %e, "is-local hook failed"),
    }
}

// ---------------------------------------------------------------- vt call counter

extern "system" {
    fn MH_CreateHook(target: *mut core::ffi::c_void, detour: *mut core::ffi::c_void, orig: *mut *mut core::ffi::c_void) -> i32;
    fn MH_QueueEnableHook(target: *mut core::ffi::c_void) -> i32;
    fn MH_ApplyQueued() -> i32;
    fn MH_EnableHook(target: *mut core::ffi::c_void) -> i32;
}
const RVA_PLAYERINS_VT: usize = 0x2a2b338;
const PLAYERINS_SLOTS: usize = 253;
static VTC_SLOT_OF: Mutex<Vec<(usize, usize)>> = Mutex::new(Vec::new()); // (slot, target rva)
static VTC_LAST: Mutex<Option<(Vec<u64>, Vec<u64>)>> = Mutex::new(None);
static VTC_REPORTS: AtomicU64 = AtomicU64::new(0);

/// Run 57: hook every distinct PlayerIns virtual method with a counting stub
/// (vtcount.rs) and report, per second, which ones run for the Hero but not
/// for the clone. `SEKIRO_COOP_VTCOUNT=1`. Installed once, on the game thread.
unsafe fn vtcount_install(hero: usize, clone: usize) {
    if std::env::var("SEKIRO_COOP_VTCOUNT").map(|v| v != "1").unwrap_or(true) {
        return;
    }
    let base = MODULE_BASE.load(Ordering::Relaxed);
    crate::vtcount::VTC_HERO = hero;
    crate::vtcount::VTC_CLONE = clone;
    let stubs = crate::vtcount::stubs();
    let vt = base + RVA_PLAYERINS_VT;
    let mut seen = std::collections::HashSet::new();
    let mut created = Vec::new();
    let (mut skipped, mut failed) = (0, 0);
    for slot in 2..PLAYERINS_SLOTS.min(crate::vtcount::N) {
        let Some(f) = read_checked::<usize>(vt + slot * 8) else { continue };
        if !seen.insert(f) || !readable(f, 16) {
            skipped += 1;
            continue;
        }
        // Too short to patch: a ret within the first 5 bytes.
        let head = core::slice::from_raw_parts(f as *const u8, 6);
        if head[..5].iter().any(|b| *b == 0xc3 || *b == 0xc2) {
            skipped += 1;
            continue;
        }
        let mut orig: *mut core::ffi::c_void = core::ptr::null_mut();
        let st = MH_CreateHook(f as *mut _, stubs[slot] as usize as *mut _, &mut orig);
        if st != 0 || orig.is_null() {
            failed += 1;
            continue;
        }
        crate::vtcount::VTC_ORIG[slot] = orig as usize;
        created.push((slot, f));
    }
    // Run 57: queued enable reported OK but no call was ever counted. Enable
    // one by one and keep the status codes.
    let mut enable_fail = 0;
    let mut first_err = None;
    for (_, f) in &created {
        let st = MH_EnableHook(*f as *mut _);
        if st != 0 {
            enable_fail += 1;
            first_err.get_or_insert(st);
        }
    }
    let _ = (MH_QueueEnableHook as usize, MH_ApplyQueued as usize);
    // Is the patch really there? First byte of a hooked function should be E9 (jmp).
    let sample: Vec<String> = created.iter().take(5).map(|(_, f)| format!("{:#x}:{:02x}", f - base, *(*f as *const u8))).collect();
    tracing::info!(hooked = created.len(), skipped, failed, enable_fail, first_err = ?first_err,
        hero = format!("{hero:#x}"), clone = format!("{clone:#x}"), sample = ?sample, "vtcount: PlayerIns methods hooked");
    *VTC_SLOT_OF.lock() = created.iter().map(|(s, f)| (*s, f - base)).collect();
}

fn vtcount_report() {
    let slots = VTC_SLOT_OF.lock().clone();
    if slots.is_empty() {
        return;
    }
    let (h, c): (Vec<u64>, Vec<u64>) = unsafe {
        (
            (0..crate::vtcount::N).map(|i| core::ptr::read_volatile(&crate::vtcount::VTC_H[i])).collect(),
            (0..crate::vtcount::N).map(|i| core::ptr::read_volatile(&crate::vtcount::VTC_C[i])).collect(),
        )
    };
    let mut last = VTC_LAST.lock();
    if let Some((lh, lc)) = last.as_ref() {
        let n = VTC_REPORTS.fetch_add(1, Ordering::Relaxed);
        if n < 12 {
            let mut hero_only = Vec::new();
            let mut clone_only = Vec::new();
            let mut both = Vec::new();
            for (slot, rva) in &slots {
                let dh = h[*slot] - lh[*slot];
                let dc = c[*slot] - lc[*slot];
                if dh > 0 && dc == 0 {
                    hero_only.push(format!("{:#x}@{rva:#x}:{dh}", slot * 8));
                } else if dc > 0 && dh == 0 {
                    clone_only.push(format!("{:#x}@{rva:#x}:{dc}", slot * 8));
                } else if dh > 0 {
                    both.push(format!("{:#x}:{dh}/{dc}", slot * 8));
                }
            }
            tracing::info!(hero_only = ?hero_only, "vtcount hero-only (slot@fn:calls/s)");
            tracing::info!(clone_only = ?clone_only, both = ?both, "vtcount clone-only and both (hero/clone)");
        }
    }
    *last = Some((h, c));
}

// ---------------------------------------------------------------- net player

/// Wolf-looking puppet (offline, 1.06): `0xa49460(WorldChrMan, id, data)` →
/// `0xa07960(WorldChrMan+0x40 slots, id, data)` takes the first free player
/// slot from index 1 (0 = local) and builds a `PlayerIns` (0xa05a80 → ctor
/// 0xa217a0), which copies `[data+8]` into `PlayerIns+0x2000` (its
/// PlayerGameData, guess from DS3 layout). Called from the net code at
/// 0xa7a840. `SEKIRO_COOP_SPAWN_PLAYER=1` creates one with the local
/// player's PlayerGameData (`[Hero+0x2000]`), so it wears the same gear.
const RVA_CREATE_NET_PLAYER: usize = 0xa49460;
const CREATE_NET_PLAYER_BYTES: [u8; 11] = [0x40, 0x53, 0x48, 0x83, 0xec, 0x20, 0x48, 0x83, 0xc1, 0x40, 0xe8];
const RVA_GAME_DATA_MAN: usize = 0x3d5aac0;
type CreateNetPlayerFn = unsafe extern "system" fn(wcm: usize, id: u32, data: *const u64) -> usize;
static SPAWN_PLAYER: Lazy<bool> = Lazy::new(|| {
    std::env::var("SEKIRO_COOP_SPAWN_PLAYER").map(|v| v == "1").unwrap_or(false)
});
static NET_PLAYER_DATA: Mutex<[u64; 16]> = Mutex::new([0; 16]);

fn spawn_player_step() {
    if !*SPAWN_PLAYER || SPAWN_DONE.load(Ordering::Relaxed) || hero_pos().is_none() {
        return;
    }
    {
        let mut first = SPAWN_FIRST_SEEN.lock();
        let t = *first.get_or_insert_with(std::time::Instant::now);
        if t.elapsed() < std::time::Duration::from_secs(10) {
            return;
        }
    }
    SPAWN_DONE.store(true, Ordering::Relaxed);
    let base = MODULE_BASE.load(Ordering::Relaxed);
    unsafe {
        let wcm = read_ptr(base + RVA_WORLD_CHR_MAN);
        let hero = wcm.and_then(|w| read_ptr(w + 0x88));
        let pgd_hero = hero.and_then(|h| read_ptr(h + 0x2000));
        let pgd_gdm = read_ptr(base + RVA_GAME_DATA_MAN).and_then(|g| read_ptr(g + 0x08));
        let slots = wcm.and_then(|w| read_checked::<i32>(w + 0x40));
        let f = base + RVA_CREATE_NET_PLAYER;
        let bytes_ok = readable(f, CREATE_NET_PLAYER_BYTES.len())
            && core::slice::from_raw_parts(f as *const u8, CREATE_NET_PLAYER_BYTES.len())
                == CREATE_NET_PLAYER_BYTES;
        tracing::info!(
            wcm = ?wcm.map(|v| format!("{v:#x}")),
            hero = ?hero.map(|v| format!("{v:#x}")),
            pgd_hero = ?pgd_hero.map(|v| format!("{v:#x}")),
            pgd_gamedataman = ?pgd_gdm.map(|v| format!("{v:#x}")),
            slot_count = ?slots,
            bytes_ok,
            "net player: state before create"
        );
        let (Some(wcm), Some(pgd)) = (wcm, pgd_hero) else {
            tracing::warn!("net player: missing WorldChrMan or PlayerGameData, not creating");
            return;
        };
        if !bytes_ok || Some(pgd) != pgd_gdm {
            tracing::warn!("net player: checks failed (bytes or PlayerGameData mismatch), not creating");
            return;
        }
        // Run 33 crashed at 0x803bfd: chr->vt[+0x308]() returned PlayerIns+0x21f8,
        // which the ctor copies from data+0x18 (we passed 0). Data layout read
        // from the ctor 0xa217a0: +0 int (id), +4 int manipulator kind
        // (0x7f8480: 0 Chr, 1 Pad, 2 Network, 3 Replay, 5 NetAI), +8
        // PlayerGameData, +0x10 optional, +0x18 object kept at +0x21f8, +0x20 int
        // kept at +0x2260. Fill +0x18/+0x20 from the Hero's own values.
        let obj21f8 = read_ptr(hero.unwrap_or(0) + 0x21f8);
        let v2260 = read_checked::<u32>(hero.unwrap_or(0) + 0x2260);
        let vt21f8 = obj21f8.and_then(|o| read_checked::<usize>(o)).map(|v| v.wrapping_sub(base));
        let manip_vt = hero
            .and_then(|h| read_ptr(h + 0x58))
            .and_then(|m| read_checked::<usize>(m))
            .map(|v| v.wrapping_sub(base));
        tracing::info!(
            obj21f8 = ?obj21f8.map(|v| format!("{v:#x}")),
            obj21f8_vtable_rva = ?vt21f8.map(|v| format!("{v:#x}")),
            v2260 = ?v2260,
            hero_manipulator_vtable_rva = ?manip_vt.map(|v| format!("{v:#x}")),
            "net player: hero fields"
        );
        let Some(obj21f8) = obj21f8 else {
            tracing::warn!("net player: Hero+0x21f8 is null, not creating");
            return;
        };
        let kind = std::env::var("SEKIRO_COOP_NET_PLAYER_KIND").ok().and_then(|v| v.parse::<u64>().ok()).unwrap_or(0);
        let ptr = {
            let mut d = NET_PLAYER_DATA.lock();
            d[0] = 1 | (kind << 32); // id 1, manipulator kind
            d[1] = pgd as u64;
            d[2] = 0;
            d[3] = obj21f8 as u64;
            d[4] = v2260.unwrap_or(0) as u64;
            d.as_ptr()
        };
        let create: CreateNetPlayerFn = core::mem::transmute(f);
        let chr = create(wcm, 1, ptr);
        let handle = if chr != 0 { read_checked::<u32>(chr + 0x08) } else { None };
        let char_id = if chr != 0 { read_checked::<u32>(chr + 0x68) } else { None };
        let vt = if chr != 0 { read_checked::<usize>(chr).map(|v| v.wrapping_sub(base)) } else { None };
        tracing::info!(
            chr = format!("{chr:#x}"),
            vtable_rva = ?vt.map(|v| format!("{v:#x}")),
            handle = ?handle,
            char_id = ?char_id,
            "net player: created"
        );
        // Run 35: the PlayerIns turned but never animated. The game's own net
        // path (0xa7a840) does more after 0xa49460: `0x9f5170(chr, id)` (stores
        // the chr type at ChrIns+0x70, then 0x9f64d0) and `chr->vt[+0x90]()`
        // (PlayerIns: 0xa23790). Do the same, byte/slot-checked.
        // Run 36: with these steps (type 1) it became invisible. Off by default.
        let post = std::env::var("SEKIRO_COOP_NET_PLAYER_POST").map(|v| v == "1").unwrap_or(false);
        if chr != 0 && post {
            let f_type = base + 0x9f5170;
            let type_ok = readable(f_type, 8)
                && core::slice::from_raw_parts(f_type as *const u8, 8) == [0x48, 0x83, 0xec, 0x28, 0x89, 0x51, 0x70, 0xe8];
            let chr_type: u32 = std::env::var("SEKIRO_COOP_NET_PLAYER_TYPE").ok().and_then(|v| v.parse().ok()).unwrap_or(1);
            if type_ok {
                let set_type: unsafe extern "system" fn(usize, u32) = core::mem::transmute(f_type);
                set_type(chr, chr_type);
            }
            let vt90_slot = read_ptr(chr).and_then(|v| read_checked::<usize>(v + 0x90));
            if vt90_slot == Some(base + 0xa23790) {
                // The net path calls vt[+0x90] first, then for types 1, 2, 12
                // clears bit 3 of ChrIns+0x1a11.
                let init: unsafe extern "system" fn(usize) = core::mem::transmute(base + 0xa23790);
                init(chr);
                if matches!(chr_type, 1 | 2 | 12) {
                    if let Some(b) = read_checked::<u8>(chr + 0x1a11) {
                        RawPtr(chr + 0x1a11).write::<u8>(b & 0xf7);
                    }
                }
            }
            let vt90_ok = vt90_slot == Some(base + 0xa23790);
            tracing::info!(
                type_ok,
                chr_type,
                vt90_ok,
                chr_type_now = ?read_checked::<u32>(chr + 0x70),
                "net player: post-create steps"
            );
        }
        if chr != 0 {
            clone_pad_install(chr);
            install_is_local_hook(chr);
            if let Some(h) = hero {
                vtcount_install(h, chr);
            }
        }
        if let (true, Some(handle), Some(char_id)) = (chr != 0, handle, char_id) {
            *TARGET.lock() = Some(Target { handle, ptr: chr, char_id });
            *LAST_WRITTEN.lock() = None;
            *PIN_TIME.lock() = Some(std::time::Instant::now());
            SPAWNED_NPC.store(u32::MAX, Ordering::Relaxed); // never auto-pin enemies
        }
    }
}

fn spawn_step() {
    spawn_player_step();
    let Some((model, npc, think)) = *SPAWN else { return };
    // After the request: pin the created character the moment it exists.
    let wait = *SPAWN_WAIT.lock();
    if let Some((creator, before, at)) = wait {
        unsafe {
            let created = read_ptr(creator + 0x1b8).unwrap_or(0);
            if created != 0 && created != before {
                let handle = read_checked::<u32>(created + 0x08);
                let char_id = read_checked::<u32>(created + 0x68);
                if let (Some(handle), Some(char_id)) = (handle, char_id) {
                    *SPAWN_WAIT.lock() = None;
                    if char_id == SPAWNED_NPC.load(Ordering::Relaxed) {
                        *TARGET.lock() = Some(Target { handle, ptr: created, char_id });
                        *LAST_WRITTEN.lock() = None;
                        PINNED_AT.store(CUR_FRAME.load(Ordering::Relaxed), Ordering::Relaxed);
                        *PIN_TIME.lock() = Some(std::time::Instant::now());
                        // Harmless at once, before its first AI update.
                        logic_off(created);
                        if let Some(team) = *TEAM {
                            RawPtr(created + 0x74).write::<u8>(team);
                        }
                        tracing::info!(
                            ptr = format!("{created:#x}"),
                            handle,
                            char_id,
                            after_ms = at.elapsed().as_millis() as u64,
                            "spawn: created character pinned"
                        );
                    } else {
                        tracing::warn!(char_id, "spawn: creator+0x1b8 changed but char id differs; not pinned");
                    }
                }
            } else if at.elapsed() > std::time::Duration::from_secs(30) {
                *SPAWN_WAIT.lock() = None;
                tracing::warn!("spawn: no created character after 30 s");
            }
        }
        return;
    }
    if SPAWN_DONE.load(Ordering::Relaxed) || hero_pos().is_none() {
        return;
    }
    // Wait until the Hero has existed for 10 s (loading finished).
    {
        let mut first = SPAWN_FIRST_SEEN.lock();
        let t = *first.get_or_insert_with(std::time::Instant::now);
        if t.elapsed() < std::time::Duration::from_secs(10) {
            return;
        }
    }
    SPAWN_DONE.store(true, Ordering::Relaxed);
    let base = MODULE_BASE.load(Ordering::Relaxed);
    let rows = npc_rows_for(model);
    tracing::info!(model, candidates = ?rows, "spawn: NpcParam rows for this model");
    let Some(npc) = npc.or_else(|| rows.first().copied()) else {
        // Show what low ids exist, to pick one by hand (c0000 has no
        // model-numbered NpcParam rows, guess).
        let low: Vec<u32> = {
            let base = MODULE_BASE.load(Ordering::Relaxed);
            unsafe {
                sekiro_sdk_sys::paramrepo::open_param(
                    sekiro_sdk_sys::paramrepo::SOLO_PARAM_REPOSITORY_RVA_V1_06,
                    base,
                    sekiro_sdk_sys::paramrepo::ParamKind::NpcParam,
                )
                .map(|t| t.rows().map(|(id, _)| id).filter(|id| *id < 10_000_000).take(200).collect())
                .unwrap_or_default()
            }
        };
        tracing::warn!(model, low_rows = ?low, "spawn: no NpcParam row for this model, not spawning");
        return;
    };
    let think = think.unwrap_or(npc);
    unsafe {
        let Some(wcm) = read_ptr(base + RVA_WORLD_CHR_MAN) else {
            tracing::warn!("spawn: no WorldChrMan");
            return;
        };
        let Some(creator) = read_ptr(wcm + OFF_CHR_CREATOR) else {
            tracing::warn!("spawn: no ChrCreator at WorldChrMan+0x3148");
            return;
        };
        let vt = read_checked::<usize>(creator);
        let ready = read_checked::<u8>(creator + 0x1b0);
        let f = base + RVA_CHR_CREATOR_REQUEST;
        let bytes_ok = readable(f, CHR_CREATOR_REQUEST_BYTES.len())
            && core::slice::from_raw_parts(f as *const u8, CHR_CREATOR_REQUEST_BYTES.len())
                == CHR_CREATOR_REQUEST_BYTES;
        tracing::info!(
            creator = format!("{creator:#x}"),
            vt_ok = vt == Some(base + RVA_CHR_CREATOR_VT),
            ready = ?ready,
            bytes_ok,
            npc,
            think,
            "spawn: creator state"
        );
        if vt != Some(base + RVA_CHR_CREATOR_VT) || !bytes_ok || !readable(creator, 0x1c0) {
            tracing::warn!("spawn: checks failed, not spawning");
            return;
        }
        if ready != Some(1) {
            tracing::warn!("spawn: creator not idle (+0x1b0 != 1); the game may not step it");
            return;
        }
        RawPtr(creator + 0x118).write::<u32>(npc);
        RawPtr(creator + 0x114).write::<u32>(think);
        RawPtr(creator + 0x1a4).write::<f32>(4.0);
        RawPtr(creator + 0x1a8).write::<f32>(0.0);
        RawPtr(creator + 0x1ac).write::<f32>(0.0);
        SPAWNED_NPC.store(npc, Ordering::Relaxed);
        let req: ChrCreatorRequestFn = core::mem::transmute(f);
        let before = read_ptr(creator + 0x1b8).unwrap_or(0);
        req(creator, model);
        *SPAWN_WAIT.lock() = Some((creator, before, std::time::Instant::now()));
        // Re-pick: the puppet should be the spawned character, not whatever
        // was pinned before.
        *TARGET.lock() = None;
        *LAST_HANDLE.lock() = None;
        *LAST_WRITTEN.lock() = None;
        tracing::info!(model, npc, think, model_at = ?read_checked::<u32>(creator + 0x1a0), "spawn: requested");
    }
}

// ---------------------------------------------------------------- mirror scan

/// Breadth-first walk from the target ChrIns through every pointer-looking
/// field (VirtualQuery-checked), logging every place a float triple equals
/// the physics-module position within 2 cm. Finds the other copies of the
/// position (physics engine proxy, render transforms) that keep reverting
/// our writes. Read-only.
fn mirror_scan(t: Target, root: usize, root_name: &str, label: &str) {
    use std::collections::{HashSet, VecDeque};
    let Some(pa) = (unsafe { phys_pos_addr(t.ptr) }) else { return };
    let p: [f32; 3] = unsafe { RawPtr(pa).read() };
    let close = |a: f32, b: f32| (a - b).abs() < 0.02;

    const MAX_NODES: usize = 4000;
    const MAX_DEPTH: usize = 3;
    let mut seen: HashSet<usize> = HashSet::new();
    let mut q: VecDeque<(usize, usize, String)> = VecDeque::new();
    q.push_back((root, 0, root_name.to_string()));
    seen.insert(root);
    let mut hits: Vec<String> = Vec::new();
    let mut nodes = 0usize;

    while let Some((obj, depth, path)) = q.pop_front() {
        nodes += 1;
        if nodes > MAX_NODES {
            break;
        }
        // ChrIns is large; nested objects get a smaller window.
        let len: usize = if depth == 0 && root == t.ptr { 0x2400 } else { 0x600 };
        let mut off = 0usize;
        while off + 12 <= len {
            let a = obj + off;
            if off % 0x200 == 0 && !readable(a, 0x200.min(len - off)) {
                off += 0x200;
                continue;
            }
            let v: [f32; 3] = unsafe { RawPtr(a).read() };
            if close(v[0], p[0]) && close(v[1], p[1]) && close(v[2], p[2]) {
                hits.push(format!("{path}+{off:#x}"));
            }
            if depth < MAX_DEPTH && off % 8 == 0 {
                let ptr: usize = unsafe { RawPtr(a).read() };
                if ptr >= 0x10000 && ptr < 0x7FFF_FFFF_0000 && ptr % 8 == 0 && !seen.contains(&ptr) {
                    if readable(ptr, 16) {
                        seen.insert(ptr);
                        q.push_back((ptr, depth + 1, format!("{path}+{off:#x}>")));
                    }
                }
            }
            off += 4;
        }
    }
    SCANS_DONE.fetch_add(1, Ordering::Relaxed);
    tracing::info!(
        scan = label,
        pos = ?p,
        nodes,
        hit_count = hits.len(),
        hits = ?hits.iter().take(60).collect::<Vec<_>>(),
        "puppet mirror scan"
    );
}

// ---------------------------------------------------------------- hero anim slots

/// The Hero's ChrIns (`[base+player_position]+0x88`).
fn hero_chr() -> Option<usize> {
    let m = crate::global()?;
    let addrs = m.base_addrs.as_ref()?;
    let base = MODULE_BASE.load(Ordering::Relaxed);
    unsafe { read_ptr(base + addrs.player_position).and_then(|a| read_ptr(a + 0x88)) }
}

/// Hero action slots (run 23): the TimeAct object `[ChrIns+0x1ff8]+0x10`
/// keeps two rings of 5 entries with stride 0x28. The ring at +0x20 holds the
/// base layer (790010 idle, 790040 combat idle, ...); the ring at +0x34 also
/// gets the attack combo (50300000, 50300010, ...), jump attack (50308000),
/// parry (50002000), jump (201001), dodge (213300). A new action is written
/// into one of the +0x34 slots, so any slot changing to a non-zero id marks a
/// new action, even when the id repeats.
static ACTION_PREV: Mutex<Option<[u32; 5]>> = Mutex::new(None);
static ACTION_CUR: Mutex<(u32, u32)> = Mutex::new((0, 0));
static ACTION_AT: Mutex<Option<std::time::Instant>> = Mutex::new(None);

pub fn hero_action() -> (u32, u32) {
    let Some(chr) = hero_chr() else { return *ACTION_CUR.lock() };
    let slots: Option<[u32; 5]> = unsafe {
        read_ptr(chr + 0x1ff8).and_then(|sm| read_ptr(sm + 0x10)).and_then(|ta| {
            if !readable(ta + 0x34, 0x28 * 4 + 4) {
                return None;
            }
            let mut v = [0u32; 5];
            for (k, x) in v.iter_mut().enumerate() {
                *x = core::ptr::read_volatile((ta + 0x34 + k * 0x28) as *const u32);
            }
            Some(v)
        })
    };
    let Some(slots) = slots else { return *ACTION_CUR.lock() };
    let mut prev = ACTION_PREV.lock();
    let mut cur = ACTION_CUR.lock();
    if let Some(p) = *prev {
        for k in 0..5 {
            let v = slots[k];
            if v != p[k] && v != 0 && v != u32::MAX {
                // Run 24: one action is written into several slots over ~100 ms;
                // count it once. The same id again after 300 ms is a new action.
                let mut at = ACTION_AT.lock();
                let recent = at
                    .map(|t| t.elapsed() < std::time::Duration::from_millis(300))
                    .unwrap_or(false);
                if v != cur.0 || !recent {
                    *cur = (v, cur.1.wrapping_add(1));
                    *at = Some(std::time::Instant::now());
                }
            }
        }
    }
    *prev = Some(slots);
    *cur
}

/// Run 22: the snapshot's animation id (`+0x1ff8 → +0x10 → +0x20`) did not
/// change between the hits of a combo, so attacks play on another slot.
/// `SEKIRO_COOP_PUPPET_SLOTSCAN=1` watches u32 fields of a few Hero objects
/// and logs every change of an animation-like value (6+ digits).
static SLOTSCAN: Lazy<bool> = Lazy::new(|| {
    std::env::var("SEKIRO_COOP_PUPPET_SLOTSCAN").map(|v| v == "1").unwrap_or(false)
});
const SLOT_WORDS: usize = 0x400 / 4;
static SLOT_PREV: Mutex<Vec<Vec<u32>>> = Mutex::new(Vec::new());
static SLOT_SEEN: Mutex<Vec<(usize, usize, Vec<u32>, u32)>> = Mutex::new(Vec::new());
static SLOT_LOGGED: AtomicU64 = AtomicU64::new(0);

fn anim_like(v: u32) -> bool {
    (100_000..=99_999_999).contains(&v)
}

fn slot_scan(frame: u64) {
    if !*SLOTSCAN {
        return;
    }
    let Some(chr) = hero_chr() else { return };
    // Objects: 0 = sm+0x10, 1 = sm+0x28 (animation), 2 = sm+0x58 (anim request),
    // 3 = chr+0x20 (request slot written by ForcePlayAnimation).
    let objs: [Option<usize>; 4] = unsafe {
        let sm = read_ptr(chr + 0x1ff8);
        [
            sm.and_then(|sm| read_ptr(sm + 0x10)),
            sm.and_then(|sm| read_ptr(sm + 0x28)),
            sm.and_then(|sm| read_ptr(sm + 0x58)),
            read_ptr(chr + 0x20),
        ]
    };
    let mut prev = SLOT_PREV.lock();
    if prev.len() != objs.len() {
        *prev = vec![Vec::new(); objs.len()];
    }
    let t_ms = T0.elapsed().as_millis() as u64;
    for (oi, o) in objs.iter().enumerate() {
        let Some(o) = *o else { continue };
        if !readable(o, SLOT_WORDS * 4) {
            continue;
        }
        let cur: Vec<u32> = unsafe {
            core::slice::from_raw_parts(o as *const u32, SLOT_WORDS).to_vec()
        };
        if prev[oi].len() == SLOT_WORDS {
            for w in 0..SLOT_WORDS {
                let (a, b) = (prev[oi][w], cur[w]);
                if a != b && (anim_like(a) || anim_like(b)) {
                    let mut seen = SLOT_SEEN.lock();
                    let e = match seen.iter_mut().position(|e| e.0 == oi && e.1 == w * 4) {
                        Some(i) => &mut seen[i],
                        None => {
                            seen.push((oi, w * 4, Vec::new(), 0));
                            seen.last_mut().unwrap()
                        }
                    };
                    if !e.2.contains(&b) && e.2.len() < 40 {
                        e.2.push(b);
                    }
                    e.3 += 1;
                    // Counters and timers change every frame: stop logging them.
                    let noisy = e.3 > 150;
                    drop(seen);
                    if !noisy && SLOT_LOGGED.fetch_add(1, Ordering::Relaxed) < 1500 {
                        tracing::info!(t_ms, obj = oi, off = format!("{:#x}", w * 4), from = a, to = b, "hero slot");
                    }
                }
            }
        }
        prev[oi] = cur;
    }
    if frame % 600 == 0 {
        let seen = SLOT_SEEN.lock();
        for (oi, off, vals, n) in seen.iter() {
            tracing::info!(obj = oi, off = format!("{off:#x}"), changes = n, values = ?vals, "hero slot summary");
        }
    }
}

// ---------------------------------------------------------------- survey

/// Called from `on_frame` (ticker thread) every frame.
pub fn on_tick(m: &crate::Mod, frame: u64) {
    if !*ENABLED {
        return;
    }
    clone_diff_step();
    bvar_scan_step();
    let _ = TICKER_TID.set(format!("{:?}", std::thread::current().id()));
    CUR_FRAME.store(frame, Ordering::Relaxed);
    write_step(true);
    slot_scan(frame);

    // Mirror scans: pause writes 2 s after pinning, scan 1 s later ("A"),
    // again 10 s after that ("B") with the same pause, so hits present in
    // both scans are stable fields rather than coincidences.
    // Copy the target out so the lock is not held during a scan (the game
    // thread takes TARGET on every SetFlag call).
    let tgt = if *SCAN { *TARGET.lock() } else { None };
    if let Some(t) = tgt {
        let since = frame.saturating_sub(PINNED_AT.load(Ordering::Relaxed));
        // "C": from the ChrIns, as before. "P": from the physics module
        // object (ChrIns+0x1ff8 → +0x68), to reach the physics engine's copy.
        for (start, label) in [(120u64, "C"), (720u64, "P")] {
            if since == start {
                PAUSE_UNTIL.store(frame + 60, Ordering::Relaxed);
            }
            if since == start + 60 {
                if label == "C" {
                    mirror_scan(t, t.ptr, "chr", label);
                } else {
                    let phys = unsafe {
                        read_ptr(t.ptr + 0x1ff8).and_then(|sm| read_ptr(sm + 0x68))
                    };
                    if let Some(ph) = phys {
                        mirror_scan(t, ph, "phys", label);
                    }
                }
            }
        }
    }
    // Remote animation transitions and the result of each forced action.
    {
        let evs: Vec<_> = std::mem::take(&mut *ANIM_EVENTS.lock());
        for (ms, from, to, moving, forced) in evs {
            if ANIM_EVENTS_LOGGED.fetch_add(1, Ordering::Relaxed) < 600 {
                tracing::info!(t_ms = ms as u64, from, to, moving, forced, rest = ?*REST_ANIM.lock(), "puppet remote anim");
            }
        }
        let due = FORCE_CHECK
            .lock()
            .filter(|(at, _)| at.elapsed() >= std::time::Duration::from_millis(300));
        if let Some((_, want)) = due {
            *FORCE_CHECK.lock() = None;
            let now_anim = (*TARGET.lock()).and_then(|t| unsafe { anim_of(t.ptr) });
            tracing::info!(want, puppet_anim = ?now_anim, "puppet forced action check");
            if now_anim == Some(want) && !CATALOG.is_empty() {
                let mut ok = CATALOG_OK.lock();
                if !ok.contains(&want) {
                    ok.push(want);
                    tracing::info!(accepted = ?*ok, "puppet catalog accepted so far");
                }
            }
        }
    }
    if frame % 60 != 0 {
        return;
    }
    vtcount_report();
    let Some(hero) = hero_pos() else { return };

    let mut freeze_now = None;
    if *FREEZE_AI {
        if let Some(addrs) = m.base_addrs.as_ref() {
            let base = MODULE_BASE.load(Ordering::Relaxed);
            let flag = base + addrs.debug_flags + 13;
            unsafe {
                if let Some(v) = read_checked::<u8>(flag) {
                    FREEZE_AI_BEFORE.lock().get_or_insert(v);
                    if v != 1 {
                        RawPtr(flag).write::<u8>(1);
                    }
                    freeze_now = read_checked::<u8>(flag);
                }
            }
        }
    }

    let entries: Vec<(u32, usize)> = m.chrins_registry.lock().iter().map(|(h, p)| (*h, *p)).collect();
    let mut rows: Vec<(f32, u32, usize, u32, u8, [f32; 3], Option<u32>)> = Vec::new();
    for (h, p) in entries {
        unsafe {
            if read_checked::<u32>(p + 0x08) != Some(h) {
                continue;
            }
            let char_id = read_checked::<u32>(p + 0x68).unwrap_or(0);
            let team = read_checked::<u8>(p + 0x74).unwrap_or(0);
            let Some(pa) = phys_pos_addr(p) else { continue };
            let pos: [f32; 3] = RawPtr(pa).read();
            rows.push((dist(pos, hero), h, p, char_id, team, pos, anim_of(p)));
        }
    }
    rows.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(core::cmp::Ordering::Equal));

    let pinned = *TARGET.lock();
    let sample: Vec<String> = rows
        .iter()
        .take(8)
        .map(|r| {
            let mark = if pinned.map(|t| t.handle) == Some(r.1) { "*" } else { "" };
            format!("{mark}c{}/t{}/h{}/d={:.1}m/anim={:?}", r.3, r.4, r.1, r.0, r.6)
        })
        .collect();
    tracing::info!(hero = ?hero, total = rows.len(), nearest = ?sample, "puppet survey");

    if pinned.is_none() {
        let last = *LAST_HANDLE.lock();
        let same = last.and_then(|(h, _)| rows.iter().find(|r| r.1 == h && r.0 < PICK_RADIUS_M));
        if let Some(r) = same {
            REPINS.fetch_add(1, Ordering::Relaxed);
            tracing::info!(handle = r.1, char_id = r.3, dist = r.0, "puppet target re-pinned (same handle)");
        }
        let pick = same.or_else(|| rows.iter().find(|r| {
            let (d, _, _, c, team, _, _) = **r;
            let spawned = SPAWNED_NPC.load(Ordering::Relaxed);
            if spawned != 0 {
                return c == spawned;
            }
            match *PIN_CHAR {
                Some(want) => c == want,
                None => team == 6 && c != 0 && d > 0.5 && d < PICK_RADIUS_M,
            }
        }));
        if let Some(r) = pick {
            let t = Target { handle: r.1, ptr: r.2, char_id: r.3 };
            *TARGET.lock() = Some(t);
            *LAST_WRITTEN.lock() = None;
            PINNED_AT.store(frame, Ordering::Relaxed);
            tracing::info!(handle = t.handle, char_id = t.char_id, dist = r.0, "puppet target pinned");
        }
    }

    let t = *TARGET.lock();
    let cur = t.and_then(|t| unsafe { phys_pos_addr(t.ptr).map(|p| RawPtr(p).read::<[f32; 3]>()) });
    // Copy every guarded value out first, one lock at a time: the game
    // thread takes LAST_WRITTEN then LAST_SEEN/HEADING, so holding several
    // of these at once here could deadlock against it.
    let team_before = *TEAM_BEFORE.lock();
    let heading = HEADING.lock().0;
    let last_written = *LAST_WRITTEN.lock();
    let last_seen = *LAST_SEEN.lock();
    let team_now = t.and_then(|t| unsafe { read_checked::<u8>(t.ptr + 0x74) });
    tracing::info!(
        writer = if *WRITER_TICKER { "ticker" } else { "game(SetFlag)" },
        writer_thread = WRITER_TID.get().map(String::as_str).unwrap_or("-"),
        ticker_thread = TICKER_TID.get().map(String::as_str).unwrap_or("-"),
        target = ?t.map(|t| (t.char_id, t.handle)),
        writes = WRITES.load(Ordering::Relaxed),
        overwritten = OVERWRITTEN.load(Ordering::Relaxed),
        stale = STALE.load(Ordering::Relaxed),
        repins = REPINS.load(Ordering::Relaxed),
        height_resets = RESCUED.load(Ordering::Relaxed),
        mode = if *MODE_RAW { "raw" } else { "warp" },
        warps = WARPS.load(Ordering::Relaxed),
        source = if *SOURCE_NET { "net" } else { "offset" },
        anim_resets = ANIM_RESETS.load(Ordering::Relaxed),
        stop_resets = STOP_RESETS.load(Ordering::Relaxed),
        anim_reset_refused = ANIM_RESET_REFUSED.load(Ordering::Relaxed),
        clone_updates = CLONE_UPDATES.load(Ordering::Relaxed),
        remote_manip_rx = REMOTE_MANIP_RX.load(Ordering::Relaxed),
        remote_manip_used = REMOTE_MANIP_USED.load(Ordering::Relaxed),
        clone_upd_calls = CLONE_UPD_CALLS.load(Ordering::Relaxed),
        subobj_copies = SUBOBJ_COPIES.load(Ordering::Relaxed),
        bvar_copies = BVAR_COPIES.load(Ordering::Relaxed),
        islocal_clone_hits = ISLOCAL_CLONE_HITS.load(Ordering::Relaxed),
        corrections = CORRECTIONS.load(Ordering::Relaxed),
        nudges = NUDGES.load(Ordering::Relaxed),
        clone_off_by = ?t.and_then(|t| unsafe { phys_pos_addr(t.ptr).map(|p| RawPtr(p).read::<[f32; 3]>()) }).zip(remote_target()).map(|(a, b)| dist([a[0], 0.0, a[2]], [b[0], 0.0, b[2]])),
        action_anim = *ACTION_ANIM,
        forces = FORCES.load(Ordering::Relaxed),
        force_refused = FORCE_REFUSED.load(Ordering::Relaxed),
        resets_held = RESETS_HELD.load(Ordering::Relaxed),
        remote_anim = ?remote_anim(),
        rest_anim = ?*REST_ANIM.lock(),
        puppet_anim = ?t.and_then(|t| unsafe { anim_of(t.ptr) }),
        yaw_deg = ?YAW.lock().map(|y| y.to_degrees()),
        puppet_rot = ?t.and_then(|t| unsafe { rot_of(t.ptr) }),
        hero_rot = ?hero_rot(),
        hero_heading = ?HEADING.lock().0,
        remote_fresh = REMOTE_FRESH_STEPS.load(Ordering::Relaxed),
        remote_stale = REMOTE_STALE_STEPS.load(Ordering::Relaxed),
        remote_pos = ?remote_target(),
        remote_yaw = ?remote_yaw(),
        hero_yaw = ?hero_yaw(),
        raw_steps = RAW_STEPS.load(Ordering::Relaxed),
        warp_ms = *WARP_MS,
        warp_refused = WARP_REFUSED.load(Ordering::Relaxed),
        logic_writes = LOGIC_WRITES.load(Ordering::Relaxed),
        logic_now = ?t.and_then(|t| unsafe { read_ptr(t.ptr + 0x50).and_then(|c| read_checked::<u8>(c + 0x190)) }),
        mirrors = ?MIRRORS.iter().map(|o| format!("{o:#x}")).collect::<Vec<_>>(),
        mirror_overwritten = ?MIRROR_OVERWRITTEN.iter().take(MIRRORS.len()).map(|c| c.load(Ordering::Relaxed)).collect::<Vec<_>>(),
        freeze_ai_before = ?*FREEZE_AI_BEFORE.lock(),
        freeze_ai_now = ?freeze_now,
        team_target = ?*TEAM,
        team_before = ?team_before,
        team_now = ?team_now,
        team_writes = TEAM_WRITES.load(Ordering::Relaxed),
        heading = ?heading,
        last_written = ?last_written,
        last_seen_by_game = ?last_seen,
        now = ?cur,
        "puppet probe stats"
    );
}
