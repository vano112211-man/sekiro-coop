//! `sekiro_coop.dll` — DLL entry point.
//!
//! Loaded by `me3`.  Wires Layers 1-5 together and owns the process-wide
//! state singleton.  SPEC Phase A exit criterion: loads with DLL
//! attached, overlay visible, every SpEffect application logged.

#![allow(clippy::missing_safety_doc)]

pub mod hooks;
pub mod overlay;
pub mod puppet;
pub mod vtcount;
pub mod tick;

use once_cell::sync::OnceCell;
use parking_lot::Mutex;
use std::net::SocketAddr;
use std::sync::Arc;

use sekiro_coop_authority::rng::{MatchSeed, SeededRng, SiteId};
use sekiro_coop_authority::table::{AuthorityTable, PeerId};
use sekiro_coop_authority::{HandoffTracker, SyncChannel};
use sekiro_coop_net::lobby::Lobby;
use sekiro_coop_net::session::{Session, SessionConfig};
use sekiro_coop_net::transport::{Transport, UdpTransport};
use sekiro_coop_net::wire::{PacketBody, PacketType, PlayerSnapshot};
use sekiro_coop_net::LinkState;
use sekiro_coop_rollback::band::SharedBand;
use sekiro_coop_rollback::ring::{Input, InputRing};
use sekiro_coop_rollback::snapshot::{RollbackSnapshot, SnapshotRing};
use sekiro_coop_rollback::stepper::ChrInsStepper;
use sekiro_coop_rollback::{Predictor, ResimDriver, SharedStepper};
use sekiro_sdk_bridge::events::BridgeDispatcher;
use sekiro_sdk_bridge::{AiBridge, CombatBridge, WorldBridge};
use sekiro_sdk_core::hook;
use sekiro_sdk_sys::chrins::ChrInsLayout;
use sekiro_sdk_sys::memory::find_current_module;
use sekiro_sdk_sys::offsets::BaseAddrs;
use sekiro_sdk_sys::version::{detect_version_live, GameVersion};
use sekiro_sdk_sys::worldchrman::WorldChrManLayout;

/// Process-wide state singleton.  Initialised on DLL attach; torn down
/// on detach.
pub struct Mod {
    pub version: GameVersion,
    pub base_addrs: Option<BaseAddrs>,
    pub chrins: ChrInsLayout,
    pub wcm: WorldChrManLayout,

    pub dispatcher: BridgeDispatcher,
    pub combat: CombatBridge,
    pub ai: AiBridge,
    pub world_bridge: WorldBridge,

    pub band: Mutex<SharedBand>,
    pub snapshots: Mutex<SnapshotRing>,
    pub local_inputs: Mutex<InputRing>,
    pub remote_inputs: Mutex<InputRing>,
    pub predictor: Predictor,
    pub resim: ResimDriver,

    pub authority: AuthorityTable,
    pub handoffs: HandoffTracker,
    pub match_seed: Mutex<MatchSeed>,
    pub frame: std::sync::atomic::AtomicU64,

    pub session: Mutex<Option<Session<Box<dyn Transport>>>>,
    pub remote_player: Mutex<Option<PlayerSnapshot>>,

    /// Set once after the first successful `WorldChrMan` scan so we
    /// don't re-run the discovery heuristic every frame.
    pub chrset_scanned: std::sync::atomic::AtomicBool,

    /// Number of consecutive frames the in-game-time counter has been
    /// *advancing* — the most reliable "world is stepping" signal.  IGT
    /// is frozen during loading screens and pause menus; if it ticks up
    /// N frames in a row we know we're actually in a live save.
    pub scan_stable_frames: std::sync::atomic::AtomicU32,
    /// Last observed IGT value for the stability counter.
    pub last_igt_ms: std::sync::atomic::AtomicU32,

    /// Best candidate from the scan.  Holds `(array_addr, hero_vtable)`;
    /// used by the periodic iteration log.  Zero means "no candidate".
    pub chrset_array_addr: std::sync::atomic::AtomicUsize,
    pub chrset_hero_vtable: std::sync::atomic::AtomicUsize,

    /// Map of `ChrIns.handle → ChrIns*`.  Populated from hook
    /// arguments (ApplyEffect); every pointer was handed to us by the
    /// game.  Keyed by handle so remote EnemyStates (which only carry
    /// handles, not local pointers) can be resolved to a local
    /// ChrIns in this instance's address space.
    pub chrins_registry: Mutex<std::collections::HashMap<u32, usize>>,

    /// Last-sent EnemyState hash per handle.  Entities whose hash is
    /// unchanged since our last broadcast are dropped from the outgoing
    /// batch — trivial delta compression for idle enemies.  Recomputed
    /// as (hp, posture, anim_id, quantized position) — a 64-bit digest
    /// is enough to skip vast majority of no-op re-sends.
    pub last_sent_hash: Mutex<std::collections::HashMap<u32, u64>>,
}

impl Mod {
    pub fn new(peer: PeerId, version: GameVersion, base_addrs: Option<BaseAddrs>) -> Self {
        Self {
            version,
            base_addrs,
            chrins: ChrInsLayout::unresolved(),
            wcm: WorldChrManLayout::unresolved(),

            dispatcher: BridgeDispatcher::new(),
            combat: CombatBridge::new(),
            ai: AiBridge::new(),
            world_bridge: WorldBridge::new(),

            band: Mutex::new(SharedBand::new()),
            snapshots: Mutex::new(SnapshotRing::with_default_capacity()),
            local_inputs: Mutex::new(InputRing::new()),
            remote_inputs: Mutex::new(InputRing::new()),
            predictor: Predictor::new(),
            resim: ResimDriver::new(),

            authority: AuthorityTable::new(peer),
            handoffs: HandoffTracker::new(),
            match_seed: Mutex::new(MatchSeed::new(0)),
            frame: std::sync::atomic::AtomicU64::new(0),

            session: Mutex::new(None),
            remote_player: Mutex::new(None),

            chrset_scanned: std::sync::atomic::AtomicBool::new(false),
            scan_stable_frames: std::sync::atomic::AtomicU32::new(0),
            last_igt_ms: std::sync::atomic::AtomicU32::new(0),
            chrset_array_addr: std::sync::atomic::AtomicUsize::new(0),
            chrset_hero_vtable: std::sync::atomic::AtomicUsize::new(0),
            chrins_registry: Mutex::new(std::collections::HashMap::with_capacity(64)),
            last_sent_hash: Mutex::new(std::collections::HashMap::with_capacity(64)),
        }
    }

    pub fn current_frame(&self) -> u64 {
        self.frame.load(std::sync::atomic::Ordering::Acquire)
    }

    pub fn advance_frame(&self) -> u64 {
        self.frame
            .fetch_add(1, std::sync::atomic::Ordering::AcqRel)
            + 1
    }

    /// Seed a deterministic RNG for a named call site at the current
    /// frame.  Mirror of SPEC §6.4.
    pub fn rng_for(&self, site: SiteId) -> SeededRng {
        SeededRng::new(*self.match_seed.lock(), self.current_frame(), site)
    }

    /// Record a local input for the current frame.
    pub fn record_local_input(&self, input: Input) {
        self.local_inputs.lock().put(input);
    }

    /// Produce the remote input for a frame, predicting if the real
    /// value hasn't arrived yet.
    pub fn remote_input_for(&self, frame: u64) -> Input {
        let ring = self.remote_inputs.lock();
        self.predictor.predict(&ring, frame)
    }

    /// Determine whether a sync channel's traffic should be sent this
    /// tick based on per-entity ownership.
    pub fn needs_state_channel(&self) -> bool {
        !self.band.lock().is_empty()
    }

    pub fn channel_reliable(&self, ch: SyncChannel) -> bool {
        ch.reliable()
    }
}

static MOD: OnceCell<Mod> = OnceCell::new();
static TICKER: OnceCell<Arc<tick::Ticker>> = OnceCell::new();

pub fn init_mod(
    peer: PeerId,
    version: GameVersion,
    base_addrs: Option<BaseAddrs>,
) -> &'static Mod {
    MOD.get_or_init(|| Mod::new(peer, version, base_addrs))
}

pub fn global() -> Option<&'static Mod> {
    MOD.get()
}

/// Perform the DLL-attach sequence.
///
/// 1. Detect Sekiro version.
/// 2. Load base addresses.
/// 3. Init MinHook.
/// 4. Install Phase-A hook (SetSpeffect) for overlay logging.
///
/// Returns `Err` if any fatal step fails; the DLL stays loaded but
/// uninitialised, and the mod will no-op.
pub fn on_attach() -> anyhow::Result<()> {
    // Log to a file next to the DLL; me3 swallows our stderr, so
    // stdout/stderr tracing would be invisible.  Writable path picked
    // from LOCALAPPDATA; falls back to tempdir.
    let log_dir = std::env::var_os("LOCALAPPDATA")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(std::env::temp_dir)
        .join("sekiro-coop");
    let _ = std::fs::create_dir_all(&log_dir);
    let log_path = log_dir.join("sekiro-coop.log");
    let log_file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&log_path)
        .ok();

    let env_filter = tracing_subscriber::EnvFilter::try_from_env("SEKIRO_COOP_LOG")
        .unwrap_or_else(|_| "info".into());
    if let Some(f) = log_file {
        let _ = tracing_subscriber::fmt()
            .with_env_filter(env_filter)
            .with_writer(std::sync::Mutex::new(f))
            .with_ansi(false)
            .try_init();
    } else {
        let _ = tracing_subscriber::fmt()
            .with_env_filter(env_filter)
            .try_init();
    }
    tracing::info!("sekiro-coop attaching (v{}) — log: {}",
        env!("CARGO_PKG_VERSION"), log_path.display());

    let m = find_current_module("sekiro.exe")?;
    puppet::MODULE_BASE.store(m.base, std::sync::atomic::Ordering::Relaxed);
    // SAFETY: `m` describes the loaded sekiro.exe image; reading its
    // bytes is safe as long as the process is our own.
    let version = unsafe {
        let image = m.as_bytes();
        detect_version_live(image, m.size as u32)
    };
    let base_addrs = BaseAddrs::for_version(version);
    tracing::info!(
        ?version,
        size = m.size,
        base = format!("{:#x}", m.base),
        has_base_addrs = base_addrs.is_some(),
        "module loaded"
    );

    // Decide our peer role from env (host vs client) — in production
    // this comes from the UI / lobby.  Default to Host.
    let peer = match std::env::var("SEKIRO_COOP_PEER").ok().as_deref() {
        Some("client") | Some("Client") => PeerId::Client,
        _ => PeerId::Host,
    };
    let mod_ref = init_mod(peer, version, base_addrs);

    hook::init()?;

    // Scan the module image for every documented AOB, then install
    // detours on the ones we care about.  Phase-A exit criterion
    // (SPEC §10): "every SpEffect application appears in log" —
    // the `ApplyEffect` detour does exactly that.
    //
    // SAFETY: the sekiro.exe image is mapped; `as_bytes` is sound.
    let natives = unsafe {
        let image = m.as_bytes();
        sekiro_sdk_sys::natives::Natives::scan(image, m.base)
    };
    let hook_results = hooks::install(&natives);
    let installed = hook_results.iter().filter(|h| h.installed).count();
    tracing::info!(
        installed,
        total = hook_results.len(),
        "hook installation finished"
    );

    // Start the 60 Hz fallback ticker — runs until a real game-tick
    // hook replaces it.  The Present-hook path calls `ticker.kick()` to
    // tell this thread to idle when it's driving us instead.
    let ticker = TICKER.get_or_init(|| Arc::new(tick::Ticker::new()));
    ticker.start(|| on_frame());

    // Optional: start a session if the transport env is configured.
    // Spawn on a retry thread because the peer may not be up yet and
    // the handshake blocks with a 5 s timeout.  Retrying in the
    // background means launch order (host vs peer-sim) doesn't matter.
    if std::env::var("SEKIRO_COOP_BIND").is_ok() || steam_transport_requested() {
        std::thread::Builder::new()
            .name("sekiro-coop-session-init".into())
            .spawn(move || {
                let mut attempt = 0u32;
                loop {
                    attempt += 1;
                    match start_session_from_env(mod_ref) {
                        Ok(_) => return,
                        Err(e) => {
                            tracing::warn!(attempt, %e, "session start failed; retrying in 2s");
                        }
                    }
                    std::thread::sleep(std::time::Duration::from_secs(2));
                }
            })
            .ok();
    }

    // Install overlay — real ImGui panel when `overlay` feature is
    // enabled; no-op logger otherwise.
    overlay::install();

    tracing::info!(peer = ?peer, "sekiro-coop ready");
    Ok(())
}

/// Tear down hooks + flush any pending telemetry.
pub fn on_detach() {
    tracing::info!("sekiro-coop detaching");
    if let Some(t) = TICKER.get() {
        t.stop();
    }
    overlay::uninstall();
    hook::remove_all();
}

fn steam_transport_requested() -> bool {
    std::env::var("SEKIRO_COOP_TRANSPORT").map(|v| v.eq_ignore_ascii_case("steam")).unwrap_or(false)
}

/// `SEKIRO_COOP_TRANSPORT=steam` + `SEKIRO_COOP_STEAM_PEER=<SteamID64>` uses
/// Steam P2P through the game's own Steam API (no IPs, no port forwarding).
/// Otherwise raw UDP from `SEKIRO_COOP_BIND` / `SEKIRO_COOP_PEER_ADDR`.
fn make_transport_from_env() -> anyhow::Result<(Box<dyn Transport>, Lobby)> {
    if steam_transport_requested() {
        #[cfg(windows)]
        {
            let own = sekiro_coop_net::steam::imp::own_steam_id();
            let peer: u64 = match std::env::var("SEKIRO_COOP_STEAM_PEER").ok().filter(|v| !v.trim().is_empty()) {
                Some(v) => v.trim().parse()?,
                None => {
                    // Solo smoke test: prove the game's Steam API answers.
                    static SOLO: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
                    if own.is_some() && !SOLO.swap(true, std::sync::atomic::Ordering::Relaxed) {
                        tracing::info!(own = ?own, "steam: no SEKIRO_COOP_STEAM_PEER; this is your SteamID64 for your friend");
                    }
                    anyhow::bail!("SEKIRO_COOP_STEAM_PEER (friend's SteamID64) not set; own id {own:?}");
                }
            };
            let t = sekiro_coop_net::steam::imp::SteamTransport::new(peer)?;
            static LOGGED: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
            if !LOGGED.swap(true, std::sync::atomic::Ordering::Relaxed) {
                tracing::info!(own = t.own_steam_id(), peer, "steam transport ready (give your own id to your friend)");
            }
            let placeholder: SocketAddr = "0.0.0.0:0".parse()?;
            return Ok((Box::new(t), Lobby::direct_udp(placeholder, 0)));
        }
        #[cfg(not(windows))]
        anyhow::bail!("steam transport needs Windows");
    }
    let bind = std::env::var("SEKIRO_COOP_BIND")?;
    let peer_addr = std::env::var("SEKIRO_COOP_PEER_ADDR")?;
    let bind_addr: SocketAddr = bind.parse()?;
    let peer_addr: SocketAddr = peer_addr.parse()?;
    let mut t = UdpTransport::bind(bind_addr)?;
    t.set_peer(peer_addr)?;
    Ok((Box::new(t), Lobby::direct_udp(peer_addr, 0)))
}

fn start_session_from_env(m: &'static Mod) -> anyhow::Result<()> {
    // If we've already got a live session, don't redo the handshake.
    if m.session.lock().is_some() {
        anyhow::bail!("session already live");
    }
    let (t, lobby) = make_transport_from_env()?;
    let cfg = SessionConfig {
        peer: m.authority.peer,
        ..Default::default()
    };
    let mut sess = Session::new(cfg, t, lobby);
    match sess.handshake()? {
        sekiro_coop_net::session::HandshakeOutcome::Ok(seed) => {
            *m.match_seed.lock() = seed;
            *m.session.lock() = Some(sess);
            tracing::info!(?seed, "session established");
        }
        sekiro_coop_net::session::HandshakeOutcome::TimedOut => {
            anyhow::bail!("handshake timed out");
        }
        other => {
            anyhow::bail!("handshake rejected: {other:?}");
        }
    }
    Ok(())
}

/// Frame hook — called once per game tick.  Drives:
///  1. local input capture
///  2. remote input poll
///  3. misprediction detection
///  4. shared-set recompute
///  5. snapshot + dispatcher flush
///  6. heartbeat
///  7. retransmits + handoff-timeout sweeps
pub fn on_frame() {
    let Some(m) = global() else { return };
    let frame = m.advance_frame();
    // Every frame, before anything reads the registry: drop characters the
    // game has freed (death/reload/area unload). Cheap: a few VirtualQuery
    // calls per registered character.
    let pruned = puppet::prune_registry(m);
    if pruned > 0 {
        tracing::info!(pruned, "chrins registry: dropped stale entries");
    }
    puppet::on_tick(m, frame);

    // Drain bridge events for this tick BEFORE flushing subscribers —
    // a flush would push an unrelated `Tick` marker we don't need to
    // broadcast.  `drain` gives us exactly what the hook detours queued.
    let outbound_events = m.dispatcher.drain();
    m.dispatcher.flush(frame);

    // Puppet experiment — in READ-ONLY scan mode, just log the 5
    // closest team-6 enemies, their char_id, actual world position,
    // and distance from Hero.  No writes.  Tells us if our "closest
    // enemy" is even rendered (visible range ~30m).
    let puppet_experiment = std::env::var("SEKIRO_COOP_PUPPET_EXPERIMENT")
        .map(|v| v == "1")
        .unwrap_or(false);
    if puppet_experiment {
        if let Some(addrs) = m.base_addrs.as_ref() {
            if let Ok(module) =
                sekiro_sdk_sys::memory::find_current_module("sekiro.exe")
            {
                let hero_pos = unsafe {
                    sekiro_sdk_sys::live::player_position_xyz(addrs, module.base)
                };
                if let Some(hp) = hero_pos {
                    let target_xyz = [hp[0] + 3.0, hp[1], hp[2]];
                    // Pick the CLOSEST team-6 enemy to the player so the
                    // experiment is observable on-screen.
                    let reg = m.chrins_registry.lock();
                    let target = reg
                        .iter()
                        .filter_map(|(h, p)| {
                            let raw = sekiro_sdk_sys::memory::RawPtr(*p);
                            let t = unsafe { sekiro_sdk_sys::live::team_type_of(raw) };
                            if t != 6 {
                                return None;
                            }
                            let snap = unsafe {
                                sekiro_sdk_sys::live::chrins_snapshot(raw)
                            };
                            let pos = snap.position?;
                            let dx = pos[0] - hp[0];
                            let dy = pos[1] - hp[1];
                            let dz = pos[2] - hp[2];
                            let d2 = dx * dx + dy * dy + dz * dz;
                            Some((*h, *p, d2, snap.char_id))
                        })
                        .min_by(|a, b| a.2.partial_cmp(&b.2).unwrap_or(core::cmp::Ordering::Equal))
                        .map(|(h, p, _, c)| (h, p, c));
                    drop(reg);
                    if let Some((handle, ptr, char_id)) = target {
                        unsafe {
                            let raw = sekiro_sdk_sys::memory::RawPtr(ptr);
                            let wrote_pos = sekiro_sdk_sys::live::chrins_write_position(
                                raw, target_xyz,
                            );
                            let wrote_anim = sekiro_sdk_sys::live::chrins_write_animation_id(
                                raw, 790020,
                            );
                            if frame % 60 == 0 {
                                let post_snap = sekiro_sdk_sys::live::chrins_snapshot(raw);
                                tracing::info!(
                                    handle,
                                    char_id,
                                    target = ?target_xyz,
                                    actual = ?post_snap.position,
                                    anim_set = 790020,
                                    anim_actual = ?post_snap.animation_id,
                                    wrote_pos,
                                    wrote_anim,
                                    "puppet experiment"
                                );
                            }
                        }
                    }
                }
            }
        }
    }

    // One-shot WorldChrMan scan — DISABLED by default.  The heuristic
    // sweep of WorldChrMan's interior dereferences arbitrary pointer-
    // shaped values which sometimes point to unmapped memory; repeated
    // AV crashes (0xa151d, 0xa156d, 0xa194d) even with stability gates.
    //
    // The replacement path is `ChrInsRegistry` below: we accumulate
    // ChrIns pointers from hook-supplied `entity` arguments
    // (ApplyEffect/DeleteEffect), which are known-good by construction.
    //
    // Set `SEKIRO_COOP_SCAN_WORLDCHRMAN=1` if you actually want to
    // exercise the blind-sweep path — useful for development only.
    const SCAN_STABILITY_THRESHOLD: u32 = 300;
    let scanner_enabled = std::env::var("SEKIRO_COOP_SCAN_WORLDCHRMAN")
        .map(|v| v == "1")
        .unwrap_or(false);
    if scanner_enabled && !m.chrset_scanned.load(std::sync::atomic::Ordering::Relaxed) {
        if let Some(addrs) = m.base_addrs.as_ref() {
            if let Ok(module) =
                sekiro_sdk_sys::memory::find_current_module("sekiro.exe")
            {
                // Key gate: IGT (in-game time) advances ONLY while the
                // world is stepping.  Loading screens, menus, and the
                // transition out of a load all freeze IGT.  If it ticks
                // up N frames in a row, the game is actually live.
                let igt_now = unsafe {
                    sekiro_sdk_sys::live::igt_ms(addrs, module.base)
                }
                .unwrap_or(0);
                let last = m
                    .last_igt_ms
                    .swap(igt_now, std::sync::atomic::Ordering::Relaxed);
                let igt_ticking = igt_now > 0 && igt_now > last;
                if !igt_ticking {
                    m.scan_stable_frames
                        .store(0, std::sync::atomic::Ordering::Relaxed);
                } else {
                    let prior = m
                        .scan_stable_frames
                        .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                    if prior + 1 >= SCAN_STABILITY_THRESHOLD {
                unsafe {
                    let chr_sym_addr = module.base + addrs.player_position;
                    let repo: usize = (chr_sym_addr as *const usize).read();
                    if repo != 0 {
                        let hero: usize =
                            ((repo + 0x88) as *const usize).read();
                        // Belt-and-braces: the Hero's vtable lives in the
                        // sekiro.exe module (it's a compiled-in vtable).
                        // If what we're reading isn't in the module range,
                        // the struct isn't finished initialising yet — do
                        // *not* sweep its interior for pointers.
                        let hero_vtable: usize =
                            if hero != 0 { (hero as *const usize).read() } else { 0 };
                        let vtable_is_in_module = hero_vtable >= module.base
                            && hero_vtable < module.base + module.size;
                        if !vtable_is_in_module {
                            tracing::trace!(
                                hero = format!("{hero:#x}"),
                                vtable = format!("{hero_vtable:#x}"),
                                "skipping scan: hero vtable not in module range"
                            );
                            m.scan_stable_frames
                                .store(0, std::sync::atomic::Ordering::Relaxed);
                        } else if hero != 0 {
                            let rep = sekiro_sdk_sys::worldchrman_scan::scan(
                                sekiro_sdk_sys::memory::RawPtr(repo),
                                sekiro_sdk_sys::memory::RawPtr(hero),
                            );
                            m.chrset_scanned
                                .store(true, std::sync::atomic::Ordering::Relaxed);
                            if let Some(best) = rep.best() {
                                m.chrset_array_addr.store(
                                    best.array_addr,
                                    std::sync::atomic::Ordering::Relaxed,
                                );
                                m.chrset_hero_vtable.store(
                                    rep.hero_vtable,
                                    std::sync::atomic::Ordering::Relaxed,
                                );
                            }
                            tracing::info!(
                                hero = format!("{hero:#x}"),
                                worldchr = format!("{repo:#x}"),
                                hero_vtable = format!("{:#x}", rep.hero_vtable),
                                candidates = rep.candidates.len(),
                                "WorldChrMan scan complete"
                            );
                            for c in rep.candidates.iter().take(6) {
                                tracing::info!(
                                    wcm_off = format!("{:#x}", c.worldchr_offset),
                                    via = c.via_intermediary,
                                    inner_off = format!("{:#x}", c.inner_offset),
                                    array = format!("{:#x}", c.array_addr),
                                    matching = c.matching_slots,
                                    contains_hero = c.contains_hero,
                                    confidence = c.confidence,
                                    "  chrset candidate"
                                );
                            }
                        }
                    }
                }
                    }
                }
            }
        }
    }

    // Log the live player state once per second so we have a visible
    // signal the mod is actually reading game memory.  Requires both
    // the base-addresses table and the module to be known.
    if frame % 60 == 0 {
        if let Some(addrs) = m.base_addrs.as_ref() {
            if let Ok(module) =
                sekiro_sdk_sys::memory::find_current_module("sekiro.exe")
            {
                let state = unsafe {
                    sekiro_sdk_sys::live::sample_player_state(addrs, module.base)
                };
                if let (Some(hp), Some(max_hp), Some(posture), Some(anim)) =
                    (state.hp, state.max_hp, state.posture, state.current_anim)
                {
                    let pos = state.position.unwrap_or([f32::NAN; 3]);
                    let hs = hooks::stats();
                    let team = state.team_type.map(
                        sekiro_sdk_core::enums::TeamType::from_raw,
                    );
                    let char_name = state
                        .char_id
                        .map(sekiro_sdk_core::characters::name_of)
                        .unwrap_or("?");
                    let remote = m.remote_player.lock().clone();
                    let remote_summary = remote
                        .map(|r| format!(
                            "peer={:?} hp={}/{} pos=({:.1},{:.1},{:.1}) anim={}",
                            r.peer, r.hp, r.max_hp, r.position[0], r.position[1], r.position[2], r.animation_id
                        ))
                        .unwrap_or_else(|| "none".into());
                    tracing::info!(
                        frame,
                        hp,
                        max_hp,
                        posture,
                        max_posture = state.max_posture.unwrap_or(0),
                        anim,
                        pos_x = pos[0],
                        pos_y = pos[1],
                        pos_z = pos[2],
                        igt_ms = state.igt_ms.unwrap_or(0),
                        team = ?team,
                        char_name,
                        remote = %remote_summary,
                        setflag_calls = hs.setflag_calls,
                        apply_effect_calls = hs.apply_effect_calls,
                        delete_effect_calls = hs.delete_effect_calls,
                        give_item_calls = hs.give_item_calls,
                        add_experience_calls = hs.add_experience_calls,
                        warp_bonfire_calls = hs.warp_bonfire_calls,
                        "player state"
                    );

                    // Once every 10 seconds, also log param-table row
                    // counts.  Validates SoloParamRepository walk.
                    if frame % 600 == 0 {
                        let summary = unsafe {
                            sekiro_sdk_sys::paramrepo::sample_param_summary(
                                sekiro_sdk_sys::paramrepo::SOLO_PARAM_REPOSITORY_RVA_V1_06,
                                module.base,
                            )
                        };
                        for (kind, count) in summary {
                            tracing::info!(
                                param = kind.name(),
                                rows = count.unwrap_or(0),
                                "param table"
                            );
                        }
                    }

                    // ChrIns registry summary every 5s.  Per entry we
                    // read entity_id (+0x08), char_id (+0x68), team_type
                    // (+0x74) directly — safe because the hook-supplied
                    // pointer is known-valid.  Gives us the enemy
                    // taxonomy (which bosses are loaded, which are
                    // friendly NPCs) without touching WorldChrMan.
                    if frame % 300 == 0 {
                        let registry = m.chrins_registry.lock();
                        let count = registry.len();
                        let entries: Vec<usize> =
                            registry.values().copied().collect();
                        drop(registry);

                        let mut teams: std::collections::HashMap<u8, u32> =
                            std::collections::HashMap::new();
                        let mut details: Vec<String> =
                            Vec::with_capacity(entries.len().min(6));
                        for (i, ptr) in entries.iter().enumerate() {
                            let (handle, char_id, team) = unsafe {
                                let p = sekiro_sdk_sys::memory::RawPtr(*ptr);
                                (
                                    sekiro_sdk_sys::live::handle_of(p),
                                    sekiro_sdk_sys::live::char_id_of(p),
                                    sekiro_sdk_sys::live::team_type_of(p),
                                )
                            };
                            *teams.entry(team).or_insert(0) += 1;
                            if i < 6 {
                                let name =
                                    sekiro_sdk_core::characters::name_of(char_id);
                                details.push(format!(
                                    "{ptr:#x}:h{handle}/c{char_id}({name})/t{team}"
                                ));
                            }
                        }
                        tracing::info!(
                            count,
                            teams = ?teams,
                            sample = ?details,
                            "chrins registry"
                        );
                    }
                }
            }
        }
    }

    if let Some(ref sess) = *m.session.lock() {
        let _ = sess.tick_heartbeat(frame);
        let _ = sess.drive_retransmits();

        // Broadcast any bridge events produced this tick.  Reliable:
        // dropping one of these (e.g. a flag set) breaks world-state
        // sync.  Only send if there's something to send.
        if !outbound_events.is_empty() {
            let _ = sess.send_reliable(
                PacketType::Event,
                &PacketBody::BridgeEvents {
                    frame,
                    events: outbound_events.clone(),
                },
            );
        }

        // Broadcast local PlayerSnapshot every tick when we have
        // enough state to read.  SAFETY: see the state-log block below.
        if let Some(addrs) = m.base_addrs.as_ref() {
            if let Ok(module) =
                sekiro_sdk_sys::memory::find_current_module("sekiro.exe")
            {
                let state = unsafe {
                    sekiro_sdk_sys::live::sample_player_state(addrs, module.base)
                };
                if let (Some(hp), Some(max_hp), Some(posture), Some(max_posture),
                        Some(pos), Some(anim)) = (
                    state.hp,
                    state.max_hp,
                    state.posture,
                    state.max_posture,
                    state.position,
                    state.current_anim,
                ) {
                    let snap = PlayerSnapshot {
                        frame,
                        peer: m.authority.peer,
                        hp,
                        max_hp,
                        posture,
                        max_posture,
                        position: pos,
                        animation_id: anim,
                        igt_ms: state.igt_ms.unwrap_or(0),
                        yaw: puppet::hero_yaw().unwrap_or(0.0),
                        action_id: puppet::hero_action().0,
                        action_seq: puppet::hero_action().1,
                    };
                    let _ = sess.send_unreliable(
                        PacketType::State,
                        &PacketBody::PlayerSnapshot(snap),
                    );
                    if let Some(bytes) = puppet::hero_manip_bytes() {
                        let _ = sess.send_unreliable(
                            PacketType::State,
                            &PacketBody::ManipState { frame, bytes },
                        );
                    }
                }

                // Broadcast EnemyStates at 5 Hz (every 12 ticks) — the
                // per-enemy HP/position changes infrequently and UDP
                // payloads start getting big past ~40 entities.
                //
                // Authority: only the Host broadcasts enemy state.
                // Clients receive and apply, but don't originate.  This
                // eliminates the "both peers claim same enemy" race and
                // sets up the canonical co-op ownership model (host
                // owns the world; clients mirror).
                let is_host = matches!(m.authority.peer, PeerId::Host);
                if is_host && frame % 12 == 0 {
                    let entries: Vec<(u32, usize)> = {
                        let reg = m.chrins_registry.lock();
                        reg.iter().map(|(h, p)| (*h, *p)).collect()
                    };
                    let mut entities: Vec<sekiro_coop_net::wire::EnemyState> =
                        Vec::with_capacity(entries.len());
                    let total_live = entries.len();
                    let mut hashes = m.last_sent_hash.lock();
                    for (handle, ptr) in entries {
                        // Re-check right before reading: the game may have
                        // freed it since the prune at frame start.
                        if !puppet::chrins_alive(ptr, handle) {
                            continue;
                        }
                        let snap = unsafe {
                            sekiro_sdk_sys::live::chrins_snapshot(
                                sekiro_sdk_sys::memory::RawPtr(ptr),
                            )
                        };
                        // Skip entries whose state chain hasn't resolved
                        // yet (common right after entity spawn).
                        if let (Some(hp), Some(max_hp), Some(pos), Some(anim)) =
                            (snap.hp, snap.max_hp, snap.position, snap.animation_id)
                        {
                            // Cheap 64-bit delta digest: hp/posture/anim
                            // combined with quantized position (dm
                            // precision is enough — sub-dm jitter
                            // doesn't matter for the remote renderer).
                            let px = (pos[0] * 10.0) as i32;
                            let py = (pos[1] * 10.0) as i32;
                            let pz = (pos[2] * 10.0) as i32;
                            let digest = (hp as u64 & 0xFFFF)
                                | ((snap.posture.unwrap_or(0) as u64 & 0xFFFF) << 16)
                                | ((anim as u64 & 0xFFFF) << 32)
                                | (((px as u64)
                                    ^ ((py as u64) << 16)
                                    ^ ((pz as u64) << 32))
                                    & 0xFFFF_0000_0000_0000);
                            if hashes.get(&snap.handle) == Some(&digest) {
                                continue; // unchanged since last send
                            }
                            hashes.insert(snap.handle, digest);

                            entities.push(sekiro_coop_net::wire::EnemyState {
                                handle: snap.handle,
                                char_id: snap.char_id,
                                team: snap.team_type,
                                hp,
                                max_hp,
                                posture: snap.posture.unwrap_or(0),
                                max_posture: snap.max_posture.unwrap_or(0),
                                position: pos,
                                animation_id: anim,
                            });
                        }
                    }
                    drop(hashes);

                    if !entities.is_empty() {
                        let count = entities.len();
                        let _ = sess.send_unreliable(
                            PacketType::State,
                            &PacketBody::EnemyStates { frame, entities },
                        );
                        if frame % 60 == 0 {
                            tracing::info!(
                                frame,
                                count,
                                total_live,
                                skipped = total_live - count,
                                "sent EnemyStates (delta)"
                            );
                        }
                    }
                }
            }
        }

        // Handle grace-buffer flush + packet draining.
        match sess.link_state() {
            LinkState::Up => {
                if !sess.grace.is_empty() {
                    let _ = sess.flush_grace_buffer();
                }
            }
            LinkState::Suspect => {
                // Quietly wait for re-contact.
            }
            LinkState::Expired => {
                // Logged once a second, not every frame (run 11 log: 3,700 lines).
                if frame % 60 == 0 {
                    tracing::warn!("peer grace expired; waiting for the peer");
                }
                // DLL should drop the session; we leave that to a
                // higher-level supervisor.
            }
        }

        // Poll + dispatch incoming packets.
        let mut buf = vec![0u8; 64 * 1024];
        while let Ok(Some((_, body))) = sess.poll_packet(&mut buf) {
            apply_incoming(m, body);
        }
    }

    // Sweep proximity handoffs whose acks timed out; caller will
    // decide whether to retry.
    let _timed_out = m.handoffs.sweep_timeouts();
}

/// Apply one inbound packet body.  Keeps per-body logic in one place.
fn apply_incoming(m: &Mod, body: PacketBody) {
    match body {
        PacketBody::FullStateSnapshot(snap) => {
            apply_full_snapshot(m, snap);
            if let Some(ref sess) = *m.session.lock() {
                sess.grace.clear_snapshot_request();
            }
        }
        PacketBody::StateDelta(delta) => {
            // Find the baseline in our snapshot ring, apply, and feed
            // the reconstructed snapshot into the restore path.
            let ring = m.snapshots.lock();
            if let Some(base) = ring.at(delta.baseline_frame) {
                let reconstructed = delta.apply(base);
                drop(ring);
                apply_full_snapshot(m, reconstructed);
            }
        }
        PacketBody::Input(batch) => {
            let mut ring = m.remote_inputs.lock();
            for input in batch.inputs {
                ring.put(input);
            }
        }
        PacketBody::PlayerSnapshot(snap) => {
            // Store the latest remote snapshot; periodic log prints it.
            *m.remote_player.lock() = Some(snap);
            puppet::note_remote_snapshot();
        }
        PacketBody::ManipState { bytes, .. } => {
            puppet::note_remote_manip(bytes);
        }
        PacketBody::BridgeEvents { frame, events } => {
            tracing::info!(
                frame,
                count = events.len(),
                "bridge events from remote"
            );
            for ev in &events {
                tracing::debug!(?ev, "  remote event");
                hooks::apply_remote_event(ev);
            }
        }
        PacketBody::EnemyStates { frame, entities } => {
            if !entities.is_empty() {
                let alive = entities.iter().filter(|e| e.hp > 0).count();
                let bosses = entities
                    .iter()
                    .filter(|e| e.max_hp > 1000 && e.team != 1)
                    .count();

                // Handle resolution: for each remote entity, look up the
                // matching LOCAL ChrIns by handle.  Log how many matched
                // + per-match HP divergence.  Writing the delta is
                // deferred behind an apply-gate (same pattern as bridge
                // events) and a later patch will wire it up.
                // Apply gate: only write HP when explicitly enabled
                // AND we are the authority-consumer (client).  The host
                // IS the authority, so it must not accept a client's
                // enemy-state claims.  Decrement-only policy — we never
                // revive, which prevents a peer from accidentally
                // healing an enemy we've already killed.
                let apply_enemy_hp = std::env::var("SEKIRO_COOP_APPLY_REMOTE")
                    .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
                    .unwrap_or(false)
                    && matches!(m.authority.peer, PeerId::Client);

                let reg = m.chrins_registry.lock();
                let mut matched = 0usize;
                let mut read_failed = 0usize;
                let mut applied = 0usize;
                let mut unmatched = Vec::<u32>::new();
                let mut diffs = Vec::<String>::new();
                for ent in &entities {
                    match reg.get(&ent.handle) {
                        Some(&local_ptr) if !puppet::chrins_alive(local_ptr, ent.handle) => {
                            read_failed += 1;
                        }
                        Some(&local_ptr) => {
                            matched += 1;
                            let chrins_raw =
                                sekiro_sdk_sys::memory::RawPtr(local_ptr);
                            let snap = unsafe {
                                sekiro_sdk_sys::live::chrins_snapshot(chrins_raw)
                            };
                            match snap.hp {
                                Some(local_hp) if local_hp != ent.hp => {
                                    let diff = ent.hp - local_hp;
                                    if diffs.len() < 5 {
                                        diffs.push(format!(
                                            "h{}:local={} remote={} delta={}",
                                            ent.handle, local_hp, ent.hp, diff
                                        ));
                                    }
                                    // Decrement-only apply.
                                    if apply_enemy_hp && ent.hp < local_hp {
                                        let wrote = unsafe {
                                            sekiro_sdk_sys::live::chrins_write_hp(
                                                chrins_raw, ent.hp,
                                            )
                                        };
                                        if wrote {
                                            applied += 1;
                                        }
                                    }
                                }
                                Some(_) => {} // equal — no divergence
                                None => read_failed += 1,
                            }
                        }
                        None => {
                            if unmatched.len() < 5 {
                                unmatched.push(ent.handle);
                            }
                        }
                    }
                }
                drop(reg);
                tracing::info!(
                    frame,
                    count = entities.len(),
                    alive,
                    bosses,
                    matched,
                    read_failed,
                    applied,
                    unmatched_count = entities.len() - matched,
                    unmatched_sample = ?unmatched,
                    divergences = ?diffs,
                    "remote enemy states"
                );
            }
        }
        _ => {
            // Other packet types handled by the layer that cares
            // (authority, etc.).  Stub for now.
        }
    }
}

/// Apply a full snapshot via the ChrInsStepper.  Skips when the
/// ChrInsLayout is still unresolved (safety).
fn apply_full_snapshot(m: &Mod, snap: RollbackSnapshot) {
    if m.chrins.validate().is_err() {
        tracing::debug!("skipping snapshot apply: ChrInsLayout unresolved");
        return;
    }
    // Resolver: look up an entity by ID in WorldChrMan and return its ChrIns pointer.
    // Until the WorldChrMan walker AOB lands, this resolver always returns None,
    // so the stepper no-ops.
    let mut resolve = |_id: sekiro_sdk_core::entity::EntityId| {
        // TODO(P0 gap #2): implement via WorldChrMan walker once the
        // AOB is validated.  Currently returns None → stepper skips.
        None
    };
    let mut stepper = ChrInsStepper::new(m.chrins, &mut resolve);
    stepper.restore(&snap);
    tracing::debug!(frame = snap.frame, wrote = stepper.last_written_entities, "snapshot applied");
    // Record into ring for future delta-baseline references.
    m.snapshots.lock().push(snap);
}

#[cfg(target_os = "windows")]
mod dllmain {
    use windows::Win32::Foundation::{BOOL, HINSTANCE};
    use windows::Win32::System::SystemServices::{DLL_PROCESS_ATTACH, DLL_PROCESS_DETACH};

    #[no_mangle]
    pub extern "system" fn DllMain(
        _hinst: HINSTANCE,
        reason: u32,
        _reserved: *mut core::ffi::c_void,
    ) -> BOOL {
        match reason {
            DLL_PROCESS_ATTACH => {
                // Run attach on a background thread so we don't block
                // the loader.
                std::thread::spawn(|| {
                    if let Err(e) = super::on_attach() {
                        tracing::error!(%e, "attach failed");
                    }
                });
            }
            DLL_PROCESS_DETACH => {
                super::on_detach();
            }
            _ => {}
        }
        BOOL(1)
    }
}
