use rand::random;
use std::{
    collections::HashMap,
    sync::mpsc::{channel, Receiver, Sender},
    thread,
};
use tokio::task;

async fn initiate(buyer_transmit: Sender<(i32, String)>, available: &HashMap<i32, String>) {
    for (&id, name) in available.iter() {
        let name = name.clone();
        let tx = buyer_transmit.clone();
        thread::spawn(move || {
            tx.send((id, name)).unwrap();
        });
    }
}

async fn offer(
    seller_recieve: Receiver<(i32, String)>,
    seller_transmit: Sender<(String, i32)>,
    prices: &HashMap<(i32, String), i32>,
) {
    let mut request_handles = vec![];

    for recieved in seller_recieve {
        println!("Received request: {:?}", recieved);
        let price = prices[&recieved];
        let tx = seller_transmit.clone();
        request_handles.push(task::spawn(async move {
            tx.send((recieved.1, price)).unwrap();
        }));
    }

    for handle in request_handles {
        handle.await.unwrap();
    }
}

async fn decide_offer(buyer_recieve: Receiver<(String, i32)>, buyer_confirm: Sender<(String, bool)>) {
    let mut confirm_handles = vec![];
    for recieved in buyer_recieve {
        println!("Price for request {:?} is: {}", recieved.0, recieved.1);
        let tx = buyer_confirm.clone();
        confirm_handles.push(task::spawn(async move {
            tx.send((recieved.0, random())).unwrap();
        }));
    }

    for handle in confirm_handles {
        handle.await.unwrap();
    }
}

async fn confirm(seller_handle: Receiver<(String, bool)>) {
    let mut offer_handles = vec![];
    for recieved in seller_handle {
        let (name, accepted) = recieved;
        let choice = if accepted { "Accepted" } else { "Rejected" };
        println!("Request {:?} was {}", name, choice);
        if accepted {
            offer_handles.push(task::spawn(flexible_offer(name)));
        }
    }
    for handle in offer_handles {
        handle.await.unwrap();
    }
}

async fn flexible_offer(item: String) {
    let (shipper_transmit, buyer_recieve) = channel();
    let (payment_request, payment_confirm) = channel();
    shipper_transmit.send(item.clone()).unwrap();
    payment_request.send(item.clone()).unwrap();
    let ship = task::spawn(try_ship(buyer_recieve));
    let pay = task::spawn(try_pay(payment_confirm));

    match (ship.await, pay.await) {
        (Ok(_), Ok(_)) => println!("{:?} shipped and paid for", item),
        (Err(_), Ok(_)) => println!("Shipping failed"),
        (Ok(_), Err(_)) => println!("Payment failed"),
        (Err(_), Err(_)) => println!("Shipping and payment failed"),
    }
}

async fn try_ship(buyer_recieve: Receiver<String>) {
    println!("{:?} shipped", buyer_recieve.recv().unwrap());
}

async fn try_pay(payment_request: Receiver<String>) {
    println!("{:?} paid for", payment_request.recv().unwrap());
}

#[tokio::main]
async fn main() {
    let (buyer_transmit, seller_recieve) = channel();
    let (seller_transmit, buyer_recieve) = channel();
    let (buyer_confirm, seller_handle) = channel();

    let available = HashMap::from([
        (1, "Apple".to_owned()),
        (2, "Banana".to_owned()),
        (3, "Orange".to_owned()),
    ]);

    let prices = available
        .iter()
        .map(|(&id, name)| ((id, name.clone()), id * 250))
        .collect::<HashMap<_, _>>();

    let get_prices = task::spawn(async move {
        initiate(buyer_transmit, &available).await;
    });

    let get_confirmation = task::spawn(async move {
        offer(seller_recieve, seller_transmit, &prices).await;
    });

    let confirm_order = task::spawn(decide_offer(buyer_recieve, buyer_confirm));

    let finalise = task::spawn(confirm(seller_handle));

    get_prices.await.unwrap();
    get_confirmation.await.unwrap();
    confirm_order.await.unwrap();
    finalise.await.unwrap();
}
