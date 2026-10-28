//! Steam P2P transport through the game's own Steam API.  SPEC §7.1.
//!
//! Sekiro 1.06 ships an old `steam_api64.dll` (FileVersion 01.68.08.89,
//! SHA-256 fc205474…b212f) with no flat API and no ISteamNetworkingMessages.
//! It exports the classic accessors (`SteamNetworking`, `SteamUser`, ...)
//! that return C++ interface pointers; `SteamNetworking()` is
//! `SteamNetworking005`.  The game initialises Steam itself (AppID 814380),
//! so we never call `SteamAPI_Init`: we take the interfaces it already has.
//! Both players own Sekiro, so P2P packets go over Valve's relay with no
//! port forwarding.
//!
//! ISteamNetworking005 vtable (MSVC x64, `this` in rcx, CSteamID by value
//! as a u64):
//!   [0] bool SendP2PPacket(CSteamID to, const void*, u32 len, EP2PSend, int ch)
//!   [1] bool IsP2PPacketAvailable(u32* size, int ch)
//!   [2] bool ReadP2PPacket(void*, u32 cap, u32* size, CSteamID* from, int ch)
//!   [3] bool AcceptP2PSessionWithUser(CSteamID)
//!   [7] bool AllowP2PPacketRelay(bool)
//! ISteamUser016 [2] CSteamID GetSteamID() returns through a hidden pointer
//! (MSVC member function returning a class).
//!
//! No callbacks: each side is told the other's SteamID64, and sending to a
//! user also accepts that user's session, so both sides sending the handshake
//! is enough.  We call AcceptP2PSessionWithUser on the peer as well.

#[cfg(all(feature = "steam", windows))]
pub mod imp {
    use crate::transport::{Transport, TransportError};
    use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4};

    #[link(name = "kernel32")]
    extern "system" {
        fn GetModuleHandleA(name: *const u8) -> usize;
        fn GetProcAddress(module: usize, name: *const u8) -> usize;
    }

    /// Channel unlikely to collide with anything the game does (it has no
    /// online mode, but keep away from 0).
    pub const CHANNEL: i32 = 7;
    /// Largest packet Steam sends unreliably; bigger ones go reliable.
    pub const UNRELIABLE_MAX: usize = 1200;
    const SEND_UNRELIABLE: i32 = 0;
    const SEND_RELIABLE: i32 = 2;

    type AccessorFn = unsafe extern "C" fn() -> usize;
    type SendFn = unsafe extern "system" fn(usize, u64, *const u8, u32, i32, i32) -> u8;
    type AvailFn = unsafe extern "system" fn(usize, *mut u32, i32) -> u8;
    type ReadFn = unsafe extern "system" fn(usize, *mut u8, u32, *mut u32, *mut u64, i32) -> u8;
    type UserFn = unsafe extern "system" fn(usize, u64) -> u8;
    type RelayFn = unsafe extern "system" fn(usize, u8) -> u8;
    type GetIdFn = unsafe extern "system" fn(usize, *mut u64) -> *mut u64;

    pub struct SteamTransport {
        net: usize,
        peer: u64,
        own: u64,
        scratch: Vec<u8>,
        pub dropped_foreign: u64,
    }

    // The interface pointer is process-global and Steam's calls are
    // thread-safe; the transport sits behind the session's Mutex anyway.
    unsafe impl Send for SteamTransport {}

    unsafe fn slot<T: Copy>(this: usize, i: usize) -> T {
        let vt = *(this as *const usize);
        let f = *((vt + i * 8) as *const usize);
        core::mem::transmute_copy(&f)
    }

    unsafe fn accessor(module: usize, name: &[u8]) -> Option<usize> {
        let p = GetProcAddress(module, name.as_ptr());
        if p == 0 {
            return None;
        }
        let f: AccessorFn = core::mem::transmute(p);
        let i = f();
        (i != 0).then_some(i)
    }

    /// The local player's SteamID64 through the game's Steam API, without
    /// opening anything.  `None` when Steam is not initialised yet.
    pub fn own_steam_id() -> Option<u64> {
        unsafe {
            let m = GetModuleHandleA(b"steam_api64.dll\0".as_ptr());
            if m == 0 {
                return None;
            }
            let user = accessor(m, b"SteamUser\0")?;
            let get: GetIdFn = slot(user, 2);
            let mut own = 0u64;
            get(user, &mut own);
            (own != 0).then_some(own)
        }
    }

    impl SteamTransport {
        /// `peer` is the other player's SteamID64.
        pub fn new(peer: u64) -> Result<Self, TransportError> {
            unsafe {
                let m = GetModuleHandleA(b"steam_api64.dll\0".as_ptr());
                if m == 0 {
                    return Err(TransportError::Unavailable("steam_api64.dll not loaded".into()));
                }
                let net = accessor(m, b"SteamNetworking\0").ok_or_else(|| {
                    TransportError::Unavailable("SteamNetworking() is null (Steam not initialised yet?)".into())
                })?;
                let mut own = 0u64;
                if let Some(user) = accessor(m, b"SteamUser\0") {
                    let get: GetIdFn = slot(user, 2);
                    get(user, &mut own);
                }
                let relay: RelayFn = slot(net, 7);
                relay(net, 1);
                let accept: UserFn = slot(net, 3);
                accept(net, peer);
                Ok(Self { net, peer, own, scratch: Vec::new(), dropped_foreign: 0 })
            }
        }

        pub fn own_steam_id(&self) -> u64 {
            self.own
        }
        pub fn peer_steam_id(&self) -> u64 {
            self.peer
        }
    }

    impl Transport for SteamTransport {
        fn send(&mut self, bytes: &[u8]) -> Result<(), TransportError> {
            let kind = if bytes.len() <= UNRELIABLE_MAX { SEND_UNRELIABLE } else { SEND_RELIABLE };
            let ok = unsafe {
                let f: SendFn = slot(self.net, 0);
                f(self.net, self.peer, bytes.as_ptr(), bytes.len() as u32, kind, CHANNEL)
            };
            if ok == 0 {
                return Err(TransportError::Unavailable(format!(
                    "SendP2PPacket failed ({} bytes)",
                    bytes.len()
                )));
            }
            Ok(())
        }

        fn recv(&mut self, buf: &mut [u8]) -> Result<Option<usize>, TransportError> {
            loop {
                let mut size = 0u32;
                let avail = unsafe {
                    let f: AvailFn = slot(self.net, 1);
                    f(self.net, &mut size, CHANNEL)
                };
                if avail == 0 {
                    return Ok(None);
                }
                if self.scratch.len() < size as usize {
                    self.scratch.resize(size as usize, 0);
                }
                let mut got = 0u32;
                let mut from = 0u64;
                let ok = unsafe {
                    let f: ReadFn = slot(self.net, 2);
                    f(
                        self.net,
                        self.scratch.as_mut_ptr(),
                        self.scratch.len() as u32,
                        &mut got,
                        &mut from,
                        CHANNEL,
                    )
                };
                if ok == 0 {
                    return Ok(None);
                }
                if from != self.peer {
                    self.dropped_foreign += 1;
                    continue;
                }
                let n = (got as usize).min(buf.len());
                buf[..n].copy_from_slice(&self.scratch[..n]);
                return Ok(Some(n));
            }
        }

        fn set_peer(&mut self, _peer: SocketAddr) -> Result<(), TransportError> {
            Ok(())
        }

        /// Steam has no socket address; report a placeholder.
        fn local_addr(&self) -> Result<SocketAddr, TransportError> {
            Ok(SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::UNSPECIFIED, 0)))
        }
    }
}
