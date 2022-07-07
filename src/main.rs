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
        TODO: The road ahead
        - Port configuration in toml
        - Further stability improvements after live testing
        - Extended functions after live testing (e.g. "/unsubscribe_all" or "/ping")
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

                        if msg.addr == "/subscribe" || msg.addr == "/unsubscribe" {
                            // Handle subscribe/unsubscribe requests

                            // NOTE: Currently forcing subscribers to manually supply their inport due to lack of
                            //  socket port selection in python osc library (update readme if this changes)
                            let port = msg.args[1].clone().string();

                            println!("Received {}", msg.addr);

                            let osc_address = match msg.args.get(0) {
                                Some(arg) => arg.clone().string(),
                                None => None
                            };

                            if osc_address.is_some() && port.is_some() {

                                // Clear subscriptions for same client (osc_addr/ip/port)
                                // Note how this is done for subscribe as well in order to avoid duplicates
                                subscribers.lock().unwrap().retain(|sdat| sdat.socket.port().to_string() != port.clone().unwrap()
                                    && sdat.socket.ip().to_string() != addr.ip().to_string() && sdat.osc_address != osc_address.clone().unwrap());

                                // Actual subscribe logic
                                if msg.addr == "/subscribe" {

                                    // Construct a subscriber address using the caller ip and the port arg (see note regarding port)
                                    let sub_addr_result = SocketAddr::from_str(&format!("{}:{}", addr.ip(), port.unwrap()));

                                    if sub_addr_result.is_ok() {

                                        subscribers.lock().unwrap().push(SubscriberData {
                                            socket: sub_addr_result.unwrap(),
                                            osc_address: osc_address.unwrap()
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

                        let msg_buf = encoder::encode(&packet).unwrap();

                        // Send bundle to subscribers. Note the custom made up address (/bundle) used for this scenario.
                        subscribers.lock().unwrap().iter()
                            .filter(|sub| sub.osc_address == "/bundle")
                            .for_each(|sub| {
                                println!("Sending to subscriber at address {}:{}...", sub.socket.ip(), sub.socket.port());
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
