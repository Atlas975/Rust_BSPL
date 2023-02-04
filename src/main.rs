#![allow(unused_macros)]
mod protcols;

use rand::random;
use std::{collections::HashMap, sync::mpsc, thread};
macro_rules! simulate_delay {
    () => {
        if random::<bool>() {
            thread::sleep(std::time::Duration::from_millis(50));
        }
    };
}

fn initiate(buyer_transmit: mpsc::Sender<(i32, String)>, available: &HashMap<i32, String>) {
    for (&id, name) in available.iter() {
        let name = name.clone();
        let tx = buyer_transmit.clone();
        thread::spawn(move || {
            tx.send((id, name)).unwrap();
        });
    }
}

fn offer(
    seller_recieve: mpsc::Receiver<(i32, String)>,
    seller_transmit: mpsc::Sender<(String, i32)>,
    prices: &HashMap<(i32, String), i32>,
) {
    for recieved in seller_recieve {
        println!("Received request: {:?}", recieved);
        let price = prices[&recieved];
        let tx = seller_transmit.clone();
        thread::spawn(move || {
            tx.send((recieved.1, price)).unwrap();
        });
    }
}

fn decide_offer(
    buyer_recieve: mpsc::Receiver<(String, i32)>,
    buyer_confirm: mpsc::Sender<(String, bool)>,
) {
    for recieved in buyer_recieve {
        println!("Price for request {:?} is: {}", recieved.0, recieved.1);
        let tx = buyer_confirm.clone();
        thread::spawn(move || {
            tx.send((recieved.0, random::<bool>())).unwrap();
        });
    }
}

fn confirm(seller_handle: mpsc::Receiver<(String, bool)>) {
    let mut offer_handles = vec![];
    for recieved in seller_handle {
        let (name, accepted) = recieved;
        let choice = if accepted { "Accepted" } else { "Rejected" };
        println!("Request {:?} was {}", name, choice);
        if accepted {
            offer_handles.push(thread::spawn(move || {
                flexible_offer(name);
            }));
        }
    }
    offer_handles
        .into_iter()
        .for_each(|handle| handle.join().unwrap());
}

fn flexible_offer(item: String) {
    let (shipper_transmit, buyer_recieve) = mpsc::channel();
    let (payment_request, payment_confirm) = mpsc::channel();
    shipper_transmit.send(item.clone()).unwrap();
    payment_request.send(item.clone()).unwrap();

    match (
        try_ship(buyer_recieve).join(),
        try_pay(payment_confirm).join(),
    ) {
        (Ok(_), Ok(_)) => println!("{:?} shipped and paid for", item),
        (Err(_), Ok(_)) => println!("Shipping failed"),
        (Ok(_), Err(_)) => println!("Payment failed"),
        (Err(_), Err(_)) => println!("Shipping and payment failed"),
    }
}

fn try_ship(buyer_recieve: mpsc::Receiver<String>) -> thread::JoinHandle<()> {
    thread::spawn(move || {
        println!("{:?} shipped", buyer_recieve.recv().unwrap());
    })
}

fn try_pay(payment_request: mpsc::Receiver<String>) -> thread::JoinHandle<()> {
    thread::spawn(move || {
        println!("{:?} paid for", payment_request.recv().unwrap());
    })
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
        initiate(buyer_transmit, &available);
    });

    let get_confirmation = thread::spawn(move || {
        offer(seller_recieve, seller_transmit, &prices);
    });

    let confirm_order = thread::spawn(move || {
        decide_offer(buyer_recieve, buyer_confirm);
    });

    let finalise = thread::spawn(move || {
        confirm(seller_handle);
    });

    get_prices.join().unwrap();
    get_confirmation.join().unwrap();
    confirm_order.join().unwrap();
    finalise.join().unwrap();
}
