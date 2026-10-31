//! `peer-simulator` — stands in for a second Sekiro instance.
//!
//! Connects to the DLL's UDP session endpoint, completes the handshake,
//! then streams a canned `PlayerSnapshot` at 60 Hz.  The DLL's inbound
//! dispatch treats us as the remote peer and logs received snapshots.
//!
//! Usage:
//!
//! ```text
//! peer-simulator --bind 0.0.0.0:28001 --peer 127.0.0.1:28000 --as client
//! ```

use std::process::ExitCode;
use std::thread;
use std::time::{Duration, Instant};

use sekiro_coop_authority::rng::MatchSeed;
use sekiro_coop_authority::table::PeerId;
use sekiro_coop_net::lobby::Lobby;
use sekiro_coop_net::session::{HandshakeOutcome, Session, SessionConfig};
use sekiro_coop_net::transport::UdpTransport;
use sekiro_coop_net::wire::{EnemyState, HandshakePayload, PacketBody, PacketType, PlayerSnapshot};

fn usage() {
    eprintln!(
        "peer-simulator - stands in for a second Sekiro instance over UDP.

Usage:
    peer-simulator [--bind <ip:port>] [--peer <ip:port>] [--as host|client]
                   [--ticks <n>] [--rate <hz>] [--fake-events]
                   [--mirror [--mirror-delay-ms <ms>] [--mirror-offset <m>]]

    --mirror  send the DLL's own position back as the remote player,
              --mirror-delay-ms late (500) and --mirror-offset m along +X (2.0)

Defaults:
    --bind   0.0.0.0:28001
    --peer   127.0.0.1:28000
    --as     client
    --rate   60
    --ticks  0 (run forever)
"
    );
}

fn main() -> ExitCode {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_env("SEKIRO_COOP_LOG")
                .unwrap_or_else(|_| "info".into()),
        )
        .init();

    let mut bind = "0.0.0.0:28001".to_string();
    let mut peer = "127.0.0.1:28000".to_string();
    let mut role = PeerId::Client;
    let mut ticks: u64 = 0;
    let mut rate_hz: u64 = 60;
    let mut fake_events = false;
    // --mirror: replay the DLL's own position back as the remote player,
    // `mirror_delay_ms` late and `mirror_offset` metres along +X, so a puppet
    // driven by the network can be checked with one game client.
    let mut mirror = false;
    let mut mirror_delay_ms: u64 = 500;
    let mut mirror_offset: f32 = 2.0;

    let args: Vec<String> = std::env::args().collect();
    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--bind" => { i += 1; bind = args[i].clone(); }
            "--peer" => { i += 1; peer = args[i].clone(); }
            "--as" => {
                i += 1;
                role = match args[i].as_str() {
                    "host" => PeerId::Host,
                    _ => PeerId::Client,
                };
            }
            "--ticks" => { i += 1; ticks = args[i].parse().unwrap_or(0); }
            "--rate" => { i += 1; rate_hz = args[i].parse().unwrap_or(60); }
            "--fake-events" => { fake_events = true; }
            "--mirror" => { mirror = true; }
            "--mirror-delay-ms" => { i += 1; mirror_delay_ms = args[i].parse().unwrap_or(500); }
            "--mirror-offset" => { i += 1; mirror_offset = args[i].parse().unwrap_or(2.0); }
            "-h" | "--help" => { usage(); return ExitCode::SUCCESS; }
            other => { eprintln!("unknown flag: {other}"); usage(); return ExitCode::FAILURE; }
        }
        i += 1;
    }

    tracing::info!(%bind, %peer, ?role, rate_hz, "peer-simulator starting");

    let bind_addr = match bind.parse() {
        Ok(a) => a,
        Err(e) => { eprintln!("bad --bind: {e}"); return ExitCode::FAILURE; }
    };
    let peer_addr = match peer.parse() {
        Ok(a) => a,
        Err(e) => { eprintln!("bad --peer: {e}"); return ExitCode::FAILURE; }
    };

    let mut transport = match UdpTransport::bind(bind_addr) {
        Ok(t) => t,
        Err(e) => { eprintln!("bind failed: {e}"); return ExitCode::FAILURE; }
    };
    if let Err(e) = sekiro_coop_net::transport::Transport::set_peer(&mut transport, peer_addr) {
        eprintln!("set_peer failed: {e}");
        return ExitCode::FAILURE;
    }

    let cfg = SessionConfig {
        peer: role,
        mod_version: sekiro_coop_net::MOD_VERSION.into(),
        game_version: "1.06".into(),
        save_hash: 0xDEADBEEF,
        game_cycle: 0,
        ..Default::default()
    };
    let mut session = Session::new(cfg, transport, Lobby::direct_udp(peer_addr, 0));

    // Retry handshake forever with a short backoff — peer may not be
    // up yet (typical when launching host/client in either order).
    let mut attempt = 0u32;
    loop {
        attempt += 1;
        match session.handshake() {
            Ok(HandshakeOutcome::Ok(seed)) => {
                tracing::info!(?seed, attempt, "handshake ok");
                break;
            }
            Ok(other) => {
                tracing::warn!(?other, attempt, "handshake failed, retrying");
            }
            Err(e) => {
                tracing::warn!(%e, attempt, "handshake error, retrying");
            }
        }
        thread::sleep(Duration::from_secs(2));
    }

    let tick_period = Duration::from_nanos(1_000_000_000 / rate_hz.max(1));
    let mut frame: u64 = 0;
    let t0 = Instant::now();
    let _unused_seed = MatchSeed::new(0); // suppress unused-import warning

    // Last EnemyStates we received from the DLL.  We echo it back with
    // a small HP perturbation to exercise the DLL's handle-lookup /
    // divergence-log path — proves the round-trip without needing a
    // second real Sekiro instance.
    let mut last_rx_enemies: Vec<EnemyState> = Vec::new();
    let mut history: std::collections::VecDeque<(Instant, [f32; 3], u32, f32, u32, u32)> =
        std::collections::VecDeque::new();
    // Mirrored input state (ManipState), replayed with the same delay.
    let mut manip_history: std::collections::VecDeque<(Instant, Vec<u8>)> =
        std::collections::VecDeque::new();
    // Mirrored DP_REGULAR packets, replayed with the same delay.
    let mut dp_history: std::collections::VecDeque<(Instant, u32, Vec<u8>, u64)> =
        std::collections::VecDeque::new();
    let mut enemy_history: std::collections::VecDeque<(Instant, u64, u16, u32, Vec<u8>, Option<Vec<u8>>)> =
        std::collections::VecDeque::new();

    loop {
        frame += 1;

        // Canned player snapshot — slow circular walk at the origin.
        let theta = (frame as f32) * 0.05;
        let mut position = [theta.cos() * 3.0, 0.0, theta.sin() * 3.0];
        let mut animation_id = if frame % 120 < 60 { 790010 } else { 790020 };
        let mut yaw = 0.0f32;
        let mut action_id = 0u32;
        let mut action_seq = 0u32;
        if mirror {
            let cutoff = Instant::now() - Duration::from_millis(mirror_delay_ms);
            // Keep only what we still need: drop entries older than the newest
            // one that is already past the cutoff.
            while history.len() > 1 && history[1].0 <= cutoff {
                history.pop_front();
            }
            match history.front() {
                Some(&(t, p, a, y, ai, aseq)) if t <= cutoff => {
                    position = [p[0] + mirror_offset, p[1], p[2]];
                    animation_id = a;
                    yaw = y;
                    action_id = ai;
                    action_seq = aseq;
                }
                // Nothing old enough yet: send nothing rather than the canned
                // circle at the world origin.
                _ => {
                    if frame % 60 == 0 {
                        tracing::info!(frame, history = history.len(), "mirror: waiting for DLL snapshots");
                    }
                    // Keep the link alive while we wait: the DLL ends the
                    // session after its grace period if nothing arrives
                    // (run 11: silence → "peer grace expired").
                    let _ = session.tick_heartbeat(frame);
                    thread::sleep(tick_period);
                    poll_only(&mut session, &mut history, mirror, role);
                    continue;
                }
            }
        }
        let snap = PlayerSnapshot {
            frame,
            peer: role,
            hp: 420,
            max_hp: 420,
            posture: 0,
            max_posture: 100,
            position,
            animation_id,
            igt_ms: t0.elapsed().as_millis() as u32,
            yaw,
            action_id,
            action_seq,
        };

        if mirror {
            let cutoff = Instant::now() - Duration::from_millis(mirror_delay_ms);
            let mut due: Option<Vec<u8>> = None;
            while manip_history.front().map(|(t, _)| *t <= cutoff).unwrap_or(false) {
                due = manip_history.pop_front().map(|(_, b)| b);
            }
            if let Some(bytes) = due {
                let _ = session.send_unreliable(
                    PacketType::State,
                    &PacketBody::ManipState { frame, bytes },
                );
            }
            while enemy_history.front().map(|e| e.0 <= cutoff).unwrap_or(false) {
                if let Some((_, sent_ms, id, kind, small, big)) = enemy_history.pop_front() {
                    let _ = session.send_unreliable(
                        PacketType::State,
                        &PacketBody::EnemyRec { sent_ms, id, kind, small, big },
                    );
                }
            }
            while dp_history.front().map(|(t, _, _, _)| *t <= cutoff).unwrap_or(false) {
                if let Some((_, ty, bytes, sent)) = dp_history.pop_front() {
                    // Keep the sender's clock (`frame`) for its jitter buffer.
                    let _ = session.send_unreliable(
                        PacketType::State,
                        &PacketBody::DpRegular { frame: sent, ty, bytes },
                    );
                }
            }
        }
        if let Err(e) =
            session.send_unreliable(PacketType::State, &PacketBody::PlayerSnapshot(snap))
        {
            tracing::error!(%e, "send failed");
            return ExitCode::FAILURE;
        }

        // Synthesise a couple of bridge events every ~5 s to exercise
        // the round-trip.  One flag set + one speffect applied to a
        // fake entity handle.
        if fake_events && frame % 300 == 0 {
            use sekiro_sdk_bridge::events::BridgeEvent;
            use sekiro_sdk_bridge::world::EventFlagEvent;
            let events = vec![
                BridgeEvent::EventFlagSet(EventFlagEvent {
                    flag_id: 11_100_100,
                    state: true,
                    synced: true,
                }),
                BridgeEvent::SpEffectApplied {
                    entity: 0xFACE_FEED,
                    id: 4800,
                },
            ];
            if let Err(e) = session.send_reliable(
                PacketType::Event,
                &PacketBody::BridgeEvents { frame, events },
            ) {
                tracing::error!(%e, "event send failed");
            } else {
                tracing::info!(frame, "synthetic events sent");
            }
        }

        // Drain incoming packets (mostly Heartbeats + remote snapshots).
        let mut buf = vec![0u8; 64 * 1024];
        while let Ok(Some((header, body))) = session.poll_packet(&mut buf) {
            tracing::debug!(?header.packet_type, "rx");
            match body {
                PacketBody::Handshake(hp) => {
                    answer_handshake(&session, role, &hp);
                }
                PacketBody::ManipState { bytes, .. } if mirror => {
                    manip_history.push_back((Instant::now(), bytes));
                }
                PacketBody::Appearance { bytes } if mirror => {
                    let _ = session.send_unreliable(PacketType::State, &PacketBody::Appearance { bytes });
                }
                PacketBody::EnemyRec { sent_ms, id, kind, small, big } if mirror => {
                    enemy_history.push_back((Instant::now(), sent_ms, id, kind, small, big));
                }
                PacketBody::DpRegular { frame: sent, ty, bytes } if mirror => {
                    dp_history.push_back((Instant::now(), ty, bytes, sent));
                }
                PacketBody::PlayerSnapshot(s) => {
                    if mirror {
                        history.push_back((Instant::now(), s.position, s.animation_id, s.yaw, s.action_id, s.action_seq));
                    }
                    if frame % 60 == 0 {
                        tracing::info!(
                            frame,
                            remote_pos = ?s.position,
                            remote_hp = s.hp,
                            "heard from DLL"
                        );
                    }
                }
                PacketBody::EnemyStates { frame: f, entities } => {
                    let bosses = entities.iter().filter(|e| e.max_hp > 1000).count();
                    let alive = entities.iter().filter(|e| e.hp > 0).count();
                    if frame % 120 == 0 {
                        tracing::info!(
                            remote_frame = f,
                            count = entities.len(),
                            alive,
                            bosses,
                            "EnemyStates from DLL"
                        );
                    }
                    last_rx_enemies = entities;
                }
                _ => {}
            }
        }

        // Echo the DLL's enemy states back once per second with each
        // HP decremented by 1.  The DLL's inbound handler should log
        // matched=N with |delta|=1 for every entity — confirms the
        // handle-keyed registry + divergence path works end-to-end.
        if frame % 60 == 0 && !last_rx_enemies.is_empty() {
            let perturbed: Vec<EnemyState> = last_rx_enemies
                .iter()
                .map(|e| EnemyState {
                    hp: (e.hp - 1).max(0),
                    ..*e
                })
                .collect();
            let count = perturbed.len();
            if let Err(e) = session.send_unreliable(
                PacketType::State,
                &PacketBody::EnemyStates {
                    frame,
                    entities: perturbed,
                },
            ) {
                tracing::error!(%e, "enemy states send failed");
            } else {
                tracing::info!(frame, count, "echoed EnemyStates back (HP-1)");
            }
        }

        let _ = session.tick_heartbeat(frame);
        if frame % 60 == 0 {
            tracing::info!(frame, pos = ?snap.position, mirror, "sent snapshot");
        }

        if ticks != 0 && frame >= ticks {
            tracing::info!("done");
            return ExitCode::SUCCESS;
        }

        thread::sleep(tick_period);
    }
}

/// Receive-only pass used while the mirror has no history yet: keeps the
/// session alive and fills the history from the DLL's snapshots.
fn poll_only<T: sekiro_coop_net::transport::Transport>(
    session: &mut Session<T>,
    history: &mut std::collections::VecDeque<(Instant, [f32; 3], u32, f32, u32, u32)>,
    mirror: bool,
    role: PeerId,
) {
    let mut buf = vec![0u8; 64 * 1024];
    while let Ok(Some((_header, body))) = session.poll_packet(&mut buf) {
        match body {
            PacketBody::Handshake(hp) => answer_handshake(session, role, &hp),
            PacketBody::PlayerSnapshot(s) if mirror => {
                history.push_back((Instant::now(), s.position, s.animation_id, s.yaw, s.action_id, s.action_seq));
            }
            _ => {}
        }
    }
}

/// The DLL re-sends its Handshake until it receives ours. Our single
/// handshake can be lost when it is sent before the DLL's socket is bound
/// (run 12: os error 10054, then the DLL timed out forever while we thought
/// the session was up). Answer every retry with our payload again.
fn answer_handshake<T: sekiro_coop_net::transport::Transport>(
    session: &Session<T>,
    role: PeerId,
    remote: &HandshakePayload,
) {
    let reply = HandshakePayload {
        mod_version: sekiro_coop_net::MOD_VERSION.into(),
        game_version: "1.06".into(),
        save_hash: 0xDEADBEEF,
        game_cycle: 0,
        match_seed: remote.match_seed,
        peer: role,
    };
    match session.send_reliable(PacketType::Handshake, &PacketBody::Handshake(reply)) {
        Ok(_) => tracing::info!("answered a handshake retry from the DLL"),
        Err(e) => tracing::warn!(%e, "handshake reply failed"),
    }
}
