use std::error::Error;
use std::fmt::format;
use rosc::encoder;
use std::net::{SocketAddrV4, SocketAddr, UdpSocket};
use std::str::FromStr;
use std::sync::{Arc, Mutex};
use rosc::OscPacket;

struct SubscriberData {
    socket: SocketAddr,
    osc_address: String
}

fn main() {
    /*
        TODO: PLAN

        1. The core functionality is to maintain a set of subscribers that receive all messages for a given address
        2. As such we do (a) receive all osc, (b) register /subscribe messages, and (c) forward by-address to subscribers
        3. /subscribe_bundle will have to be its own separate logic; there are no indications that bundles can have addresses

        Getting started:
        1. Use the examples to ready a listener that prints incoming OSC info
        2. Prepare a simple python script for sending OSC to this application (can be in this repo /python)
        3. Create a branching of logic for /subscribe vs regular messages (start with just printing differently)
        4. Create a subscribers list and prepare basic forwarding logic using either inherent or provided address data
        5. Create a python application that subscribes on start so that you can test forwarding to that
            - Can also be in this repo /python
            - *** You are here. We now have working subscribe logic.
        6. This should take care of the core logic. Continue with the following additional features:
            - Unsubscribe, unsubscribe all
            - Error proofing (resubscribe, subscribed to subscribe, etc)
            - Bundles subscribing
            - Port configuration in toml
            - Is threading needed or simply inherent in the osc receiver?
            - Readme and publish
     */

    let subscriber_data: Vec<SubscriberData> = vec![];
    let subscribers = Arc::new(Mutex::new(subscriber_data));

    let local_addr = match SocketAddrV4::from_str("127.0.0.1:13339") {
        Ok(addr) => addr,
        Err(err) => panic!("{}", err.description()),
    };

    let sock = UdpSocket::bind(local_addr).unwrap();
    println!("Listening to {}", local_addr);

    let mut buf = [0u8; rosc::decoder::MTU];

    loop {

        match sock.recv_from(&mut buf) {
            Ok((size, addr)) => {
                println!("Received packet with size {} from: {}", size, addr);
                let (_, packet) = rosc::decoder::decode_udp(&buf[..size]).unwrap();
                match packet.clone() {
                    OscPacket::Message(msg) => {

                        println!("OSC address: {}", msg.addr);
                        println!("OSC arguments: {:?}", msg.args);

                        if msg.addr == "/subscribe" {
                            // Handle subscribe requests

                            // NOTE: Currently forcing subscribers to manually supply their inport due to lack of
                            //  socket port selection in python library
                            let port = msg.args[1].clone().string();

                            println!("Received subscribe");

                            let osc_address = match msg.args.get(0) {
                                Some(arg) => arg.clone().string(),
                                None => None
                            };

                            if osc_address.is_some() && port.is_some() {

                                // Avoid duplicate subscription calls for same client (addr/port)
                                subscribers.lock().unwrap().retain(|sdat| sdat.socket.port().to_string() != port.clone().unwrap()
                                    && sdat.socket.ip().to_string() != addr.ip().to_string());

                                // Construct a subscriber address using the caller ip and the port arg (see note regarding port)
                                let sub_addr = match SocketAddr::from_str(&format!("{}:{}", addr.ip(), port.unwrap())) {
                                    Ok(addr) => addr,
                                    Err(err) => panic!("{}", err.description()),
                                };

                                subscribers.lock().unwrap().push(SubscriberData {
                                    socket: sub_addr,
                                    osc_address: msg.args[0].clone().string().unwrap()
                                });


                            } else {
                                println!("WARN: Malformed subscribe message - either address or port missing");
                            }

                        } else {

                            let msg_buf = encoder::encode(&packet).unwrap();

                            // Handle regular messages that are to be sent to subscribers
                            subscribers.lock().unwrap().iter()
                                .filter(|sub| sub.osc_address == msg.addr)
                                .for_each(|sub| {
                                    println!("Sending to subscriber at address {}:{}...", sub.socket.ip(), sub.socket.port());
                                    sock.send_to(&msg_buf, sub.socket);
                                });
                        }

                    }
                    OscPacket::Bundle(bundle) => {
                        println!("OSC Bundle: {:?}", bundle);
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
