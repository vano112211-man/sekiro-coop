//! Wire protocol — packet framing, types, reliability scheme.  SPEC §9.

use sekiro_coop_authority::rng::MatchSeed;
use sekiro_coop_authority::table::PeerId;
use sekiro_coop_rollback::delta::SnapshotDelta;
use sekiro_coop_rollback::ring::Input;
use sekiro_coop_rollback::snapshot::RollbackSnapshot;
use serde::{Deserialize, Serialize};

pub const WIRE_MAGIC: u32 = 0x5345_4B52; // "SEKR" big-endian
pub const WIRE_VERSION: u8 = 1;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u8)]
pub enum PacketType {
    Input = 0x01,
    State = 0x02,
    Event = 0x03,
    BarrierRequest = 0x04,
    BarrierAck = 0x05,
    Handoff = 0x06,
    HandoffAck = 0x07,
    FullStateSnapshot = 0x08,
    DesyncReport = 0x09,
    Heartbeat = 0x0A,
    Handshake = 0x0B,
    Quit = 0x0C,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct Seq(pub u32);

impl Seq {
    pub fn next(self) -> Seq {
        Seq(self.0.wrapping_add(1))
    }
}

/// 32-preceding-ack bitmap for the sliding-window reliability layer.
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub struct AckBits(pub u32);

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct PacketHeader {
    pub magic: u32,
    pub version: u8,
    pub packet_type: PacketType,
    /// bit0 = reliable, bit1 = ordered.
    pub flags: u8,
    pub seq: Seq,
    pub ack: Seq,
    pub ack_bits: AckBits,
}

impl PacketHeader {
    pub fn reliable(&self) -> bool {
        self.flags & 0b01 != 0
    }
    pub fn ordered(&self) -> bool {
        self.flags & 0b10 != 0
    }

    pub fn new(ty: PacketType, seq: Seq, reliable: bool, ordered: bool) -> Self {
        let mut flags = 0u8;
        if reliable {
            flags |= 0b01;
        }
        if ordered {
            flags |= 0b10;
        }
        Self {
            magic: WIRE_MAGIC,
            version: WIRE_VERSION,
            packet_type: ty,
            flags,
            seq,
            ack: Seq(0),
            ack_bits: AckBits(0),
        }
    }
}

/// All packet bodies, sum type.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PacketBody {
    /// Local player inputs for a range of frames.
    Input(InputBatch),
    /// Compressed per-entity delta.
    State(StateBatch),
    /// Snapshot delta vs a specific baseline frame.
    StateDelta(SnapshotDelta),
    /// Per-tick player-state snapshot.  First real cross-peer payload.
    PlayerSnapshot(PlayerSnapshot),
    /// Batch of BridgeEvents produced during a single tick.
    BridgeEvents { frame: u64, events: Vec<sekiro_sdk_bridge::events::BridgeEvent> },
    /// Batch of non-player ChrIns states — enemies, NPCs, mini-bosses.
    /// Unreliable: the next tick overwrites it anyway.  Trimmed on the
    /// sender to fit the UDP datagram limit.
    EnemyStates { frame: u64, entities: Vec<EnemyState> },
    /// Discrete events for this tick.
    Event(EventBatch),
    BarrierRequest { name: String, deadline_ms: u32 },
    BarrierAck { name: String },
    Handoff(HandoffPayload),
    HandoffAck { entity_id: u32, new_owner: PeerId },
    FullStateSnapshot(RollbackSnapshot),
    DesyncReport { frame: u64, my_hash: u64 },
    Heartbeat { remote_frame: u64, ping_tag: u32 },
    Handshake(HandshakePayload),
    Quit { reason: String },
    /// The sender's processed player input (its PadManipulator state, run 38
    /// layout): the receiver's clone PlayerIns plays the same moves.
    ManipState { frame: u64, bytes: Vec<u8> },
    /// A packet the sender's game handed to its PlayerSession (`Send(buf,
    /// size, ty)`; ty 1 = DP_REGULAR from 0x8da080, 7 also seen); the receiver
    /// gives it back to its clone's session `Receive` for the same type.
    DpRegular { frame: u64, ty: u32, bytes: Vec<u8> },
    /// The sender's ChrAsm equip ids and face (PlayerGameData copies), so
    /// the receiver's clone can look like the sender.
    Appearance { bytes: Vec<u8> },
    /// One enemy's sync records from the host's game (stage 2): `small` is
    /// the 0x1c record (position, rotation), `big` the 0x158 behaviour
    /// record when it changed. `sent_ms` is the sender's clock.
    EnemyRec { sent_ms: u64, id: u16, small: Vec<u8>, big: Option<Vec<u8>> },
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct PlayerSnapshot {
    pub frame: u64,
    pub peer: PeerId,
    pub hp: i32,
    pub max_hp: i32,
    pub posture: i32,
    pub max_posture: i32,
    pub position: [f32; 3],
    pub animation_id: u32,
    pub igt_ms: u32,
    /// Facing, radians, as the game stores it in the physics module
    /// (`+0x70` = `[0, yaw, 0, 0]`). Lets the remote puppet turn in place.
    pub yaw: f32,
    /// Latest action animation (attack, jump, dodge, ...) from the Hero's
    /// action slots (TimeAct `+0x34 + k*0x28`, run 23), and a counter that
    /// goes up each time one starts, so a repeated id is still seen as new.
    pub action_id: u32,
    pub action_seq: u32,
}

/// One non-player ChrIns at a given tick.  Identified by `handle`,
/// which is the entity-handle the game assigns to every loaded ChrIns
/// (stable across frames for the life of the entity).
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct EnemyState {
    /// ChrIns handle (ChrIns+0x08).  Authoritative identity.
    pub handle: u32,
    /// Character-id (c-number, e.g. `c3000` for Gyoubu).
    pub char_id: u32,
    /// Team (`TeamType` raw byte).
    pub team: u8,
    pub hp: i32,
    pub max_hp: i32,
    pub posture: i32,
    pub max_posture: i32,
    pub position: [f32; 3],
    pub animation_id: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HandshakePayload {
    pub mod_version: String,
    pub game_version: String,
    pub save_hash: u64,
    pub game_cycle: u8,
    pub match_seed: MatchSeed,
    pub peer: PeerId,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InputBatch {
    pub start_frame: u64,
    pub inputs: Vec<Input>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateBatch {
    pub frame: u64,
    pub entities: Vec<StateDelta>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct StateDelta {
    pub entity_id: u32,
    /// Quantised position (1cm resolution, signed cm from world origin).
    pub pos_q_cm: [i32; 3],
    /// smallest-three 32-bit quaternion compression.
    pub rot_q: u32,
    /// half-float velocity (packed into 2 bytes per axis).
    pub vel_half: [u16; 3],
    pub animation_id: u32,
    pub animation_frame_q: u16, // 0..65535 → 0.0..1.0
    pub hp: i32,
    pub posture: u16, // quantised f32 * 100
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventBatch {
    pub frame: u64,
    pub events: Vec<sekiro_sdk_bridge::events::BridgeEvent>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HandoffPayload {
    pub entity_id: u32,
    pub new_owner: PeerId,
    pub snapshot: sekiro_coop_rollback::snapshot::EntitySnapshot,
}

// Ensure the pulled-in bridge types are actually in scope for `EventBatch`.
// sekiro-coop-net depends on authority + rollback, but we also need the
// bridge event type. We take a transitive dep through rollback →
// authority, and add bridge directly here for the event-batch type.
//
// Keep external deps at the workspace root; the `Cargo.toml` for this
// crate pulls in `sekiro-sdk-bridge` transitively through
// `sekiro-coop-rollback`.

// Compress a quat into 32 bits via smallest-three.  Keep the
// arithmetic centralised so both peers serialise identically.
pub fn pack_quat(q: [f32; 4]) -> u32 {
    let mut largest_idx = 0usize;
    let mut largest_abs = q[0].abs();
    for (i, v) in q.iter().enumerate().skip(1) {
        if v.abs() > largest_abs {
            largest_abs = v.abs();
            largest_idx = i;
        }
    }
    let sign = if q[largest_idx] < 0.0 { -1.0 } else { 1.0 };
    // 10 bits per component for the three smaller values, signed.
    let mut comps = [0i16; 3];
    let mut ci = 0;
    for (i, v) in q.iter().enumerate() {
        if i == largest_idx {
            continue;
        }
        let scaled = (sign * v * 511.0).clamp(-511.0, 511.0) as i16;
        comps[ci] = scaled;
        ci += 1;
    }
    ((largest_idx as u32 & 0b11) << 30)
        | (((comps[0] as u32) & 0x3FF) << 20)
        | (((comps[1] as u32) & 0x3FF) << 10)
        | ((comps[2] as u32) & 0x3FF)
}

pub fn unpack_quat(p: u32) -> [f32; 4] {
    let largest_idx = ((p >> 30) & 0b11) as usize;
    let x = sign_extend_10((p >> 20) & 0x3FF) as f32 / 511.0;
    let y = sign_extend_10((p >> 10) & 0x3FF) as f32 / 511.0;
    let z = sign_extend_10(p & 0x3FF) as f32 / 511.0;
    let sum = (x * x + y * y + z * z).min(1.0);
    let w = (1.0 - sum).sqrt();
    let mut out = [0f32; 4];
    let mut si = 0;
    for i in 0..4 {
        if i == largest_idx {
            out[i] = w;
        } else {
            match si {
                0 => out[i] = x,
                1 => out[i] = y,
                _ => out[i] = z,
            }
            si += 1;
        }
    }
    out
}

fn sign_extend_10(v: u32) -> i16 {
    let v = v & 0x3FF;
    if v & 0x200 != 0 {
        (v | !0x3FF) as i16
    } else {
        v as i16
    }
}

pub fn pack_half(f: f32) -> u16 {
    let bits = f.to_bits();
    let sign = ((bits >> 16) & 0x8000) as u16;
    let exp = ((bits >> 23) & 0xFF) as i32 - 127 + 15;
    let mant = (bits >> 13) & 0x3FF;
    let exp = exp.clamp(0, 0x1F) as u16;
    sign | (exp << 10) | mant as u16
}

pub fn unpack_half(h: u16) -> f32 {
    let sign = (h & 0x8000) as u32;
    let exp = ((h >> 10) & 0x1F) as i32;
    let mant = (h & 0x3FF) as u32;
    if exp == 0 && mant == 0 {
        return f32::from_bits(sign << 16);
    }
    let bits = (sign << 16) | (((exp - 15 + 127) as u32) << 23) | (mant << 13);
    f32::from_bits(bits)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn quat_roundtrip() {
        let q = [0.7071, 0.0, 0.7071, 0.0];
        let packed = pack_quat(q);
        let r = unpack_quat(packed);
        for i in 0..4 {
            assert!((q[i] - r[i]).abs() < 0.02, "{:?} vs {:?}", q, r);
        }
    }
}
