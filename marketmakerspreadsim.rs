use std::net::{UdpSocket, SocketAddr};
use std::time::Duration;
use rand::Rng;

fn main() {
    let market_maker_socket = UdpSocket::bind("127.0.0.1:8080").expect("Couldn't bind to address");
    market_maker_socket.set_read_timeout(Some(Duration::from_secs(1))).expect("Couldn't set timeout");

    let client_socket = UdpSocket::bind("127.0.0.1:8081").expect("Couldn't bind to address");
    let market_maker_addr: SocketAddr = "127.0.0.1:8080".parse().unwrap();

    let mut rng = rand::thread_rng();
    loop {
        let bid_price = rng.gen_range(95..100);
        let ask_price = rng.gen_range(100..105);

        let message = format!("BID:{} ASK:{}", bid_price, ask_price);
        client_socket.send_to(message.as_bytes(), market_maker_addr).expect("Couldn't send data");

        let mut buffer = [0; 1024];
        match market_maker_socket.recv_from(&mut buffer) {
            Ok((size, addr)) => {
                let received = String::from_utf8_lossy(&buffer[..size]);
                let parts: Vec<&str> = received.split_whitespace().collect();
                if let [bid, ask] = parts[..] {
                    let bid_price: i32 = bid[4..].parse().unwrap();
                    let ask_price: i32 = ask[4..].parse().unwrap();

                    if bid_price + 1 < ask_price {
                        let profit = ask_price - bid_price;
                        println!("Profit made: {} units from {}", profit, addr);
                    } else {
                        println!("No profitable spread available");
                    }
                }
            }
            Err(_) => continue,
        }
    }
}

