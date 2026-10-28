//! Transport abstraction — raw UDP impl; Steam P2P impl behind the
//! `steam` feature.  SPEC §7.1.

use std::io;
use std::net::{SocketAddr, UdpSocket};
use std::time::Duration;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum TransportError {
    #[error(transparent)]
    Io(#[from] io::Error),
    #[error("transport closed")]
    Closed,
    #[error("peer address unknown")]
    NoPeer,
    #[error("timeout")]
    Timeout,
    #[error("transport unavailable: {0}")]
    Unavailable(String),
}

pub trait Transport: Send {
    /// Send bytes to the configured peer.
    fn send(&mut self, bytes: &[u8]) -> Result<(), TransportError>;

    /// Non-blocking receive.  Returns `Ok(None)` when no packet is ready.
    fn recv(&mut self, buf: &mut [u8]) -> Result<Option<usize>, TransportError>;

    /// Set the remote peer address/socket.
    fn set_peer(&mut self, peer: SocketAddr) -> Result<(), TransportError>;

    fn local_addr(&self) -> Result<SocketAddr, TransportError>;
}

impl Transport for Box<dyn Transport> {
    fn send(&mut self, bytes: &[u8]) -> Result<(), TransportError> {
        (**self).send(bytes)
    }
    fn recv(&mut self, buf: &mut [u8]) -> Result<Option<usize>, TransportError> {
        (**self).recv(buf)
    }
    fn set_peer(&mut self, peer: SocketAddr) -> Result<(), TransportError> {
        (**self).set_peer(peer)
    }
    fn local_addr(&self) -> Result<SocketAddr, TransportError> {
        (**self).local_addr()
    }
}

/// Datagram-based transport using `std::net::UdpSocket`.
pub struct UdpTransport {
    socket: UdpSocket,
    peer: Option<SocketAddr>,
}

impl UdpTransport {
    pub fn bind(addr: SocketAddr) -> Result<Self, TransportError> {
        let socket = UdpSocket::bind(addr)?;
        socket.set_nonblocking(true)?;
        socket.set_read_timeout(Some(Duration::from_millis(0))).ok();
        Ok(Self { socket, peer: None })
    }
}

impl Transport for UdpTransport {
    fn send(&mut self, bytes: &[u8]) -> Result<(), TransportError> {
        let peer = self.peer.ok_or(TransportError::NoPeer)?;
        self.socket.send_to(bytes, peer)?;
        Ok(())
    }

    fn recv(&mut self, buf: &mut [u8]) -> Result<Option<usize>, TransportError> {
        match self.socket.recv_from(buf) {
            Ok((n, from)) => {
                // Accept the first sender or the configured peer.
                if self.peer.is_none() {
                    self.peer = Some(from);
                }
                Ok(Some(n))
            }
            Err(e) if e.kind() == io::ErrorKind::WouldBlock => Ok(None),
            Err(e) => Err(TransportError::Io(e)),
        }
    }

    fn set_peer(&mut self, peer: SocketAddr) -> Result<(), TransportError> {
        self.peer = Some(peer);
        Ok(())
    }

    fn local_addr(&self) -> Result<SocketAddr, TransportError> {
        Ok(self.socket.local_addr()?)
    }
}
