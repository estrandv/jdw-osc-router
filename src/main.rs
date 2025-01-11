use rosc::encoder;
use rosc::OscPacket;
use std::error::Error;
use std::fmt::format;
use std::net::{SocketAddr, SocketAddrV4, UdpSocket};
use std::str::FromStr;
use std::sync::{Arc, Mutex};

struct SubscriberData {
    socket: SocketAddr,
    osc_address: String,
}

fn main() {
    /*
       TODO: The road ahead
       - Port configuration and default subscriptions in toml
       - Handle unsubscribe
       - Ideally: List subscriptions endpoint
    */

    let mut subscriber_data: Vec<SubscriberData> = vec![];

    let local_addr = match SocketAddrV4::from_str("127.0.0.1:13339") {
        Ok(addr) => addr,
        Err(err) => panic!("{}", err.description()),
    };

    let sock = UdpSocket::bind(local_addr).unwrap();
    println!("Listening to {}", local_addr);

    // NOTE: Still not sure if I'm using buf correctly here
    // - if decoder arg is set too low we can't read large packets...
    let mut buf = [0u8; 333000];

    loop {
        match sock.recv_from(&mut buf) {
            Ok((size, addr)) => {
                println!("Received packet with size {} from: {}", size, addr);
                let (_, packet) = rosc::decoder::decode_udp(&buf[..size]).unwrap();
                match packet.clone() {
                    OscPacket::Message(msg) => {
                        println!("OSC address: {}", msg.addr);
                        println!("OSC arguments: {:?}", msg.args);

                        if msg.addr == "/subscribe" || msg.addr == "/unsubscribe" {
                            // Handle subscribe/unsubscribe requests

                            let ip = msg.args[1].clone().string();
                            let port = msg.args[2].clone().int();

                            println!("Received {}", msg.addr);

                            let osc_address = match msg.args.get(0) {
                                Some(arg) => arg.clone().string(),
                                None => None,
                            };

                            if osc_address.is_some() && port.is_some() && ip.is_some() {
                                // Clear subscriptions for same client (osc_addr/ip/port)
                                // Note how this is done for subscribe as well in order to avoid duplicates
                                subscriber_data.retain(|sub_data| {
                                    let sub_port = sub_data.socket.port() as i32;
                                    let sub_addr = sub_data.socket.ip().to_string();
                                    let sub_osc_addr = sub_data.osc_address.clone();

                                    sub_port != port.clone().unwrap()
                                        || sub_addr != ip.clone().unwrap()
                                        || sub_osc_addr != osc_address.clone().unwrap()
                                });

                                // Actual subscribe logic
                                if msg.addr == "/subscribe" {
                                    // Construct a subscriber address using the ip and port args
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
                                    } else {
                                        println!("WARN: Unable to register socket for provided subscriber address: {}", sub_addr_result.err().unwrap())
                                    }
                                }
                            } else {
                                println!("WARN: Malformed subscribe/unsubscribe message - either address or port missing");
                            }
                        } else {
                            let msg_buf = encoder::encode(&packet).unwrap();

                            // Handle regular messages that are to be sent to subscribers
                            subscriber_data
                                .iter()
                                .filter(|sub| sub.osc_address == msg.addr)
                                .for_each(|sub| {
                                    println!(
                                        "Sending to subscriber at address {}:{}...",
                                        sub.socket.ip(),
                                        sub.socket.port()
                                    );
                                    sock.send_to(&msg_buf, sub.socket);
                                });
                        }
                    }
                    OscPacket::Bundle(bundle) => {
                        println!("OSC Bundle: {:?}", bundle);

                        let msg_buf = encoder::encode(&packet).unwrap();

                        // Send bundle to subscribers. Note the custom made up address (/bundle) used for this scenario.
                        subscriber_data
                            .iter()
                            .filter(|sub| sub.osc_address == "/bundle")
                            .for_each(|sub| {
                                println!(
                                    "Sending to subscriber at address {}:{}...",
                                    sub.socket.ip(),
                                    sub.socket.port()
                                );
                                sock.send_to(&msg_buf, sub.socket);
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
