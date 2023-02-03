

struct Buyer {
    money: u32,
    items: Vec<String>,
}

struct Seller {
    stock: HashMap<String, u32>,
    prices: HashMap<String, u32>,
}


enum Item {
    Apple = 25,
    Banana = 50,
    Orange = 75,
}


impl Seller {
    fn new() -> Self {
        let stock = HashMap::from([
            ("Apple".to_owned(), 10),
            ("Banana".to_owned(), 10),
            ("Orange".to_owned(), 10),
        ]);
        let prices = HashMap::from([
            ("Apple".to_owned(), 1),
            ("Banana".to_owned(), 2),
            ("Orange".to_owned(), 3),
        ]);
        Self { stock, prices }
    }
}
