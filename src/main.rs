use std::{collections::HashMap, sync::mpsc, thread};
use rand::random;
macro_rules! simulate_delay {
    () => {
        if random::<bool>() {
            thread::sleep(std::time::Duration::from_millis(10));
        }
    };
}
fn request_price(buyer_transmit: mpsc::Sender<(i32, String)>, available: &HashMap<i32, String>) {
    for (&id, name) in available.iter() {
        let name = name.clone();
        let tx = buyer_transmit.clone();
        thread::spawn(move || {
            simulate_delay!();
            tx.send((id, name)).unwrap();
        });
    }
}


fn send_prices(
    seller_recieve: mpsc::Receiver<(i32, String)>,
    seller_transmit: mpsc::Sender<(String, i32)>,
    prices: &HashMap<(i32, String), i32>,
) {
    for recieved in seller_recieve {
        println!("Received request: {:?}", recieved);
        let price = prices[&recieved];
        let tx = seller_transmit.clone();
        thread::spawn(move || {
            simulate_delay!();
            tx.send((recieved.1, price)).unwrap();
        });
    }
}

fn request_confirmation(buyer_recieve: mpsc::Receiver<(String, i32)>, buyer_confirm: mpsc::Sender<(String, bool)>) {
    for recieved in buyer_recieve {
        println!("Price for request {:?} is: {}", recieved.0, recieved.1);
        let tx = buyer_confirm.clone();
        thread::spawn(move || {
            simulate_delay!();
            tx.send((recieved.0, random::<bool>())).unwrap();
        });
    }
}
fn finalise_order(seller_handle: mpsc::Receiver<(String, bool)>) {
    for recieved in seller_handle {
        simulate_delay!();
        let choice = if recieved.1 { "Accepted" } else { "Rejected" };
        println!("Request {:?} was {}", recieved.0, choice);
    }
}

fn main() {
    let (buyer_transmit, seller_recieve) = mpsc::channel();
    let (seller_transmit, buyer_recieve) = mpsc::channel();
    let (buyer_confirm, seller_handle) = mpsc::channel();

    let available = HashMap::from([
        (1, "Apple".to_owned()),
        (2, "Banana".to_owned()),
        (3, "Orange".to_owned()),
    ]);

    let prices = available
        .iter()
        .map(|(&id, name)| ((id, name.clone()), id * 250))
        .collect::<HashMap<_, _>>();

    let get_prices = thread::spawn(move || {
        request_price(buyer_transmit, &available);
    });

    let get_confirmation = thread::spawn(move || {
        send_prices(seller_recieve, seller_transmit, &prices);
    });

    let confirm_order = thread::spawn(move || {
        request_confirmation(buyer_recieve, buyer_confirm);
    });

    let finalise = thread::spawn(move || {
        finalise_order(seller_handle);
    });

    get_prices.join().unwrap();
    get_confirmation.join().unwrap();
    confirm_order.join().unwrap();
    finalise.join().unwrap();
}
// Attempts to implement protocols from the BSPL paper https://citeseerx.ist.psu.edu/document?repid=rep1&type=pdf&doi=a1475c0797f309ba01945e3c4bcc541d598b766c
