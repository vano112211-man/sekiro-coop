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
static RESET_ANIM: Lazy<bool> = Lazy::new(|| {
    std::env::var("SEKIRO_COOP_PUPPET_RESET_ANIM").map(|v| v != "0").unwrap_or(true)
});
static LAST_RESET_AT: Mutex<Option<std::time::Instant>> = Mutex::new(None);
static ANIM_RESETS: AtomicU64 = AtomicU64::new(0);
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

/// Puppet rotation: the physics module keeps a quaternion at +0x70.
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
            // Yaw from the target's own movement (net) or the Hero's heading.
            let yaw = {
                let mut lt = LAST_TARGET.lock();
                let mut yg = YAW.lock();
                if let Some(prev) = *lt {
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
            let warped = !*MODE_RAW
                && (warp_due || off_by > 2.0)
                && (*YAW.lock())
                    .map(yaw_rot)
                    .or_else(|| rot_of(t.ptr))
                    .map(|r| native_warp(t.ptr, desired, r))
                    .unwrap_or(false);
            if warped {
                *LAST_WARP_AT.lock() = Some(std::time::Instant::now());
            } else {
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
        write_step(false);
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

// ---------------------------------------------------------------- survey

/// Called from `on_frame` (ticker thread) every frame.
pub fn on_tick(m: &crate::Mod, frame: u64) {
    if !*ENABLED {
        return;
    }
    let _ = TICKER_TID.set(format!("{:?}", std::thread::current().id()));
    CUR_FRAME.store(frame, Ordering::Relaxed);
    write_step(true);

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
    if frame % 60 != 0 {
        return;
    }
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
        anim_reset_refused = ANIM_RESET_REFUSED.load(Ordering::Relaxed),
        yaw_deg = ?YAW.lock().map(|y| y.to_degrees()),
        puppet_rot = ?t.and_then(|t| unsafe { rot_of(t.ptr) }),
        hero_rot = ?hero_rot(),
        hero_heading = ?HEADING.lock().0,
        remote_fresh = REMOTE_FRESH_STEPS.load(Ordering::Relaxed),
        remote_stale = REMOTE_STALE_STEPS.load(Ordering::Relaxed),
        remote_pos = ?remote_target(),
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
