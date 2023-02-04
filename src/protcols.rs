use std::{collections::HashMap, sync::mpsc::{self}, thread};

use rand::random;

trait ErrorHandle {
    fn recieve_error(error: mpsc::RecvError){
        println!("Error recieving: {:?}", error);
    }

    fn send_error<T>(error: mpsc::SendError<T>){
        println!("Error sending: {:?}", error);
    }
}

struct Buyer {
    id: u32,
    buyer_transmit: mpsc::Sender<(u32, String)>,
    buyer_recieve: mpsc::Receiver<(u32, String, u32)>,
    buyer_confirm: mpsc::Sender<(String, bool)>,
}

impl ErrorHandle for Buyer {}
impl Buyer {
    fn new(
        buyer_transmit: mpsc::Sender<(u32, String)>,
        buyer_recieve: mpsc::Receiver<(u32, String, u32)>,

        buyer_confirm: mpsc::Sender<(String, bool)>,
    ) -> Self {
        Self {
            id: random(),
            buyer_transmit,
            buyer_recieve,
            buyer_confirm,
        }
    }

    fn initiate(&self, item: String) {
        // create a RecvError with a custom message
        match self.buyer_transmit.send((self.id, item)) {
            Ok(_) => println!("Buyer {} sent request", self.id),
            Err(_) => println!("Failed request from buyer {}", self.id),
        }
    }

    fn decide_offer(&self) {
        match self.buyer_recieve.recv() {
            Ok(recieved) => {
                println!("Price for request {:?} is: {}", recieved.1, recieved.2);
                let tx = self.buyer_confirm.clone();
                tx.send((recieved.1, random::<bool>())).unwrap();
            }
            Err(_) => println!("Failed to recieve offer"),
        }
    }

    // take in an error and print it out
    fn error_handle(error: mpsc::RecvError) {
        println!("Error: {:?}", error);
    }
}

impl ErrorHandle for Seller {}
struct Seller {
    seller_recieve: mpsc::Receiver<(i32, String)>,
    seller_transmit: mpsc::Sender<(String, i32)>,
    seller_handle: mpsc::Receiver<(String, bool)>,
}

impl Seller {
    fn new(
        seller_recieve: mpsc::Receiver<(i32, String)>,
        seller_transmit: mpsc::Sender<(String, i32)>,
        seller_handle: mpsc::Receiver<(String, bool)>,
    ) -> Self {
        Self {
            seller_recieve,
            seller_transmit,
            seller_handle,
        }
    }

    fn offer(&self, prices: &HashMap<(i32, String), i32>) {
        for recieved in self.seller_recieve.iter() {
            println!("Received request: {:?}", recieved);
            let price = prices[&recieved];
            let tx = self.seller_transmit.clone();
            thread::spawn(move || {
                tx.send((recieved.1, price)).unwrap();
            });
        }
    }

    fn confirm(&self) {
        let mut offer_handles = vec![];
        for recieved in self.seller_handle {
            let (name, accepted) = recieved;
            let choice = if accepted { "Accepted" } else { "Rejected" };
            println!("Request {:?} was {}", name, choice);
            if accepted {
                offer_handles.push(thread::spawn(move || {
                    Self::flexible_offer(name);
                }));
            }
        }
        offer_handles
            .into_iter()
            .for_each(|handle| handle.join().unwrap());
    }

    // fn flexible_offer(item: String) {
    //     let (shipper_transmit, buyer_recieve) = mpsc::channel();
    //     let (payment_request, payment_confirm) = mpsc::channel();
    //     shipper_transmit.send(item.clone()).unwrap();
    //     payment_request.send(item.clone()).unwrap();

    //     match (
    //         try_ship(buyer_recieve).join(),
    //         try_pay(payment_confirm).join(),
    //     ) {
    //         (Ok(_), Ok(_)) => println!("{:?} shipped and paid for", item),
    //         (Err(_), Ok(_)) => println!("Shipping failed"),
    //         (Ok(_), Err(_)) => println!("Payment failed"),
    //         (Err(_), Err(_)) => println!("Shipping and payment failed"),
    //     }
    // }
}

// struct Dis
