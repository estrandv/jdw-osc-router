use std::error::Error;
use std::net::{SocketAddrV4, UdpSocket};
use std::str::FromStr;
use rosc::OscPacket;

fn main() {
    /*
        TODO: PLAN

        1. The core functionality is to maintain a set of subscribers that receive all messages for a given address
        2. As such we do (a) receive all osc, (b) register /subscribe messages, and (c) forward by-address to subscribers
        3. /subscribe_bundle will have to be its own separate logic; there are no indications that bundles can have addresses

        Getting started:
        1. Use the examples to ready a listener that prints incoming OSC info
            - You are here. Pasted and runs.
        2. Prepare a simple python script for sending OSC to this application (can be in this repo /python)
        3. Create a branching of logic for /subscribe vs regular messages (start with just printing differently)
        4. Create a subscribers list and prepare basic forwarding logic using either inherent or provided address data
        5. Create a python application that subscribes on start so that you can test forwarding to that
            - Can also be in this repo /python
        6. This should take care of the core logic. Continue with the following additional features:
            - Unsubscribe, unsubscribe all
            - Error proofing (resubscribe, subscribed to subscribe, etc)
            - Bundles subscribing
            - Port configuration in toml
            - Is threading needed or simply inherent in the osc receiver?
            - Readme and publish
     */


    let addr = match SocketAddrV4::from_str("127.0.0.1:13339") {
        Ok(addr) => addr,
        Err(err) => panic!("{}", err.description()),
    };

    let sock = UdpSocket::bind(addr).unwrap();
    println!("Listening to {}", addr);

    let mut buf = [0u8; rosc::decoder::MTU];

    loop {
        match sock.recv_from(&mut buf) {
            Ok((size, addr)) => {
                println!("Received packet with size {} from: {}", size, addr);
                let (_, packet) = rosc::decoder::decode_udp(&buf[..size]).unwrap();
                handle_packet(packet);
            }
            Err(e) => {
                println!("Error receiving from socket: {}", e);
                break;
            }
        }
    }

    fn handle_packet(packet: OscPacket) {
        match packet {
            OscPacket::Message(msg) => {
                println!("OSC address: {}", msg.addr);
                println!("OSC arguments: {:?}", msg.args);
            }
            OscPacket::Bundle(bundle) => {
                println!("OSC Bundle: {:?}", bundle);
            }
        }
    }

}
