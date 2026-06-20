pub mod config;

use rosc::encoder;
use rosc::OscPacket;
use std::collections::HashMap;
use std::net::{SocketAddr, UdpSocket};
use std::str::FromStr;

/// Run the OSC router. Blocks the calling thread indefinitely.
///
/// * `config_path` – path to the per-app `config.toml` (falls back to defaults + central config).
/// * `quiet`       – suppress non-error log output.
pub fn run(config_path: &str, quiet: bool) {
    let _ = simple_logger::SimpleLogger::new()
        .with_level(if quiet {
            log::LevelFilter::Error
        } else {
            log::LevelFilter::Info
        })
        .init();

    let config = config::load(config_path);
    let local_addr = config.bind_addr();
    let sock = UdpSocket::bind(local_addr).unwrap();
    log::info!("Listening to {}", local_addr);

    let mut buf = vec![0u8; config.buffer_size];
    let mut subscriber_data: HashMap<String, Vec<SocketAddr>> = HashMap::new();

    loop {
        match sock.recv_from(&mut buf) {
            Ok((size, addr)) => {
                log::debug!("Received packet with size {} from: {}", size, addr);

                let (_, packet) = rosc::decoder::decode_udp(&buf[..size]).unwrap();
                let msg_buf = encoder::encode(&packet).unwrap();

                match packet {
                    OscPacket::Message(msg) => {
                        log::debug!("OSC address: {}", msg.addr);
                        log::debug!("OSC arguments: {:?}", msg.args);

                        if msg.addr == "/subscribe" || msg.addr == "/unsubscribe" {
                            let ip = msg.args[1].clone().string();
                            let port = msg.args[2].clone().int();

                            log::debug!("Received {}", msg.addr);

                            let osc_address = match msg.args.get(0) {
                                Some(arg) => arg.clone().string(),
                                None => None,
                            };

                            if let (Some(ref addr_pattern), Some(p), Some(i)) =
                                (&osc_address, port, ip)
                            {
                                // Remove matching subscriber entry
                                if let Some(addrs) = subscriber_data.get_mut(addr_pattern) {
                                    addrs.retain(|s| {
                                        s.port() as i32 != p || s.ip().to_string() != i
                                    });
                                    if addrs.is_empty() {
                                        subscriber_data.remove(addr_pattern);
                                    }
                                }

                                if msg.addr == "/subscribe" {
                                    match SocketAddr::from_str(&format!("{}:{}", i, p)) {
                                        Ok(socket) => {
                                            subscriber_data
                                                .entry(osc_address.unwrap())
                                                .or_default()
                                                .push(socket);
                                        }
                                        Err(e) => log::warn!(
                                            "Unable to register socket for provided subscriber address: {}",
                                            e
                                        ),
                                    }
                                }
                            } else {
                                log::warn!("Malformed subscribe/unsubscribe message - either address or port missing");
                            }
                        } else {
                            if let Some(subscribers) = subscriber_data.get(&msg.addr) {
                                for sub in subscribers {
                                    log::debug!(
                                        "Sending to subscriber at address {}:{}...",
                                        sub.ip(),
                                        sub.port()
                                    );
                                    let _ = sock.send_to(&msg_buf, sub);
                                }
                            }
                        }
                    }
                    OscPacket::Bundle(_bundle) => {
                        log::debug!("OSC Bundle: {:?}", _bundle);

                        if let Some(subscribers) = subscriber_data.get("/bundle") {
                            for sub in subscribers {
                                log::debug!(
                                    "Sending to subscriber at address {}:{}...",
                                    sub.ip(),
                                    sub.port()
                                );
                                let _ = sock.send_to(&msg_buf, sub);
                            }
                        }
                    }
                }
            }
            Err(e) => {
                log::error!("Error receiving from socket: {}", e);
                break;
            }
        }
    }
}
