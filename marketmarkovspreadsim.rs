use std::net::{UdpSocket, SocketAddr};
use std::time::Duration;
use rand::Rng;
use std::collections::HashMap;

fn main() {
    let market_maker_socket = UdpSocket::bind("127.0.0.1:8080").expect("Couldn't bind to address");
    market_maker_socket.set_read_timeout(Some(Duration::from_secs(1))).expect("Couldn't set timeout");

    let client_socket = UdpSocket::bind("127.0.0.1:8081").expect("Couldn't bind to address");
    let market_maker_addr: SocketAddr = "127.0.0.1:8080".parse().unwrap();

    let mut rng = rand::thread_rng();
    let mut price_transitions: HashMap<i32, Vec<i32>> = HashMap::new();

    loop {
        let bid_price = rng.gen_range(95..100);
        let ask_price = rng.gen_range(100..105);

        price_transitions.entry(bid_price).or_insert_with(Vec::new).push(ask_price);

        let next_bid_price = predict_next_price(bid_price, &price_transitions);
        let next_ask_price = predict_next_price(ask_price, &price_transitions);

        let message = format!("BID:{} ASK:{} PREDICTED_BID:{} PREDICTED_ASK:{}",
                              bid_price, ask_price, next_bid_price, next_ask_price);
        client_socket.send_to(message.as_bytes(), market_maker_addr).expect("Couldn't send data");

        let mut buffer = [0; 1024];
        match market_maker_socket.recv_from(&mut buffer) {
            Ok((size, addr)) => {
                let received = String::from_utf8_lossy(&buffer[..size]);
                let parts: Vec<&str> = received.split_whitespace().collect();
                if let [bid, ask, predicted_bid, predicted_ask] = parts[..] {
                    let bid_price: i32 = bid[4..].parse().unwrap();
                    let ask_price: i32 = ask[4..].parse().unwrap();
                    let predicted_bid_price: i32 = predicted_bid[13..].parse().unwrap();
                    let predicted_ask_price: i32 = predicted_ask[14..].parse().unwrap();

                    if predicted_bid_price + 1 < predicted_ask_price {
                        let profit = predicted_ask_price - predicted_bid_price;
                        println!("Predicted profit: {} units from {}", profit, addr);
                    } else {
                        println!("No profitable spread available based on prediction");
                    }
                }
            }
            Err(_) => continue,
        }
    }
}

fn predict_next_price(current_price: i32, transitions: &HashMap<i32, Vec<i32>>) -> i32 {
    if let Some(possible_prices) = transitions.get(&current_price) {
        let mut rng = rand::thread_rng();
        *possible_prices.get(rng.gen_range(0..possible_prices.len())).unwrap_or(&current_price)
    } else {
        current_price
    }
}

