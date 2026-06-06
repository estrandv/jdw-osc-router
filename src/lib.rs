pub mod config;

use rosc::encoder;
use rosc::OscPacket;
use std::net::{SocketAddr, UdpSocket};
use std::str::FromStr;

struct SubscriberData {
    socket: SocketAddr,
    osc_address: String,
}

/// Run the OSC router. Blocks the calling thread indefinitely.
///
/// * `config_path` – path to the per-app `config.toml` (falls back to defaults + central config).
/// * `quiet`       – suppress non-error log output.
pub fn run(config_path: &str, quiet: bool) {
    let config = config::load(config_path);
    let local_addr = config.bind_addr();
    let sock = UdpSocket::bind(local_addr).unwrap();
    if !quiet {
        println!("Listening to {}", local_addr);
    }

    let mut buf = vec![0u8; config.buffer_size];
    let mut subscriber_data: Vec<SubscriberData> = vec![];

    loop {
        match sock.recv_from(&mut buf) {
            Ok((size, addr)) => {
                if !quiet {
                    println!("Received packet with size {} from: {}", size, addr);
                }

                let (_, packet) = rosc::decoder::decode_udp(&buf[..size]).unwrap();
                let msg_buf = encoder::encode(&packet).unwrap();

                match packet {
                    OscPacket::Message(msg) => {
                        if !quiet {
                            println!("OSC address: {}", msg.addr);
                            println!("OSC arguments: {:?}", msg.args);
                        }

                        if msg.addr == "/subscribe" || msg.addr == "/unsubscribe" {
                            let ip = msg.args[1].clone().string();
                            let port = msg.args[2].clone().int();

                            if !quiet {
                                println!("Received {}", msg.addr);
                            }

                            let osc_address = match msg.args.get(0) {
                                Some(arg) => arg.clone().string(),
                                None => None,
                            };

                            if osc_address.is_some() && port.is_some() && ip.is_some() {
                                subscriber_data.retain(|sub_data| {
                                    sub_data.socket.port() as i32 != port.clone().unwrap()
                                        || sub_data.socket.ip().to_string() != ip.clone().unwrap()
                                        || sub_data.osc_address != osc_address.clone().unwrap()
                                });

                                if msg.addr == "/subscribe" {
                                    let sub_addr_result = SocketAddr::from_str(&format!(
                                        "{}:{}",
                                        ip.unwrap(),
                                        port.unwrap()
                                    ));

                                    if sub_addr_result.is_ok() {
                                        subscriber_data.push(SubscriberData {
                                            socket: sub_addr_result.unwrap(),
                                            osc_address: osc_address.unwrap(),
                                        });
                                    } else if !quiet {
                                        println!("WARN: Unable to register socket for provided subscriber address: {}", sub_addr_result.err().unwrap())
                                    }
                                }
                            } else if !quiet {
                                println!("WARN: Malformed subscribe/unsubscribe message - either address or port missing");
                            }
                        } else {
                            subscriber_data
                                .iter()
                                .filter(|sub| sub.osc_address == msg.addr)
                                .for_each(|sub| {
                                    if !quiet {
                                        println!(
                                            "Sending to subscriber at address {}:{}...",
                                            sub.socket.ip(),
                                            sub.socket.port()
                                        );
                                    }
                                    let _ = sock.send_to(&msg_buf, sub.socket);
                                });
                        }
                    }
                    OscPacket::Bundle(_bundle) => {
                        if !quiet {
                            println!("OSC Bundle: {:?}", _bundle);
                        }

                        subscriber_data
                            .iter()
                            .filter(|sub| sub.osc_address == "/bundle")
                            .for_each(|sub| {
                                if !quiet {
                                    println!(
                                        "Sending to subscriber at address {}:{}...",
                                        sub.socket.ip(),
                                        sub.socket.port()
                                    );
                                }
                                let _ = sock.send_to(&msg_buf, sub.socket);
                            });
                    }
                }
            }
            Err(e) => {
                println!("Error receiving from socket: {}", e);
                break;
            }
        }
    }
}
