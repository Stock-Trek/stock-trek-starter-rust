use crate::algorithm::algorithm::MyAlgorithm;
use std::collections::HashMap;
use stock_trek::prelude::{Exchange, StockTrekAlgorithm, StockTrekContext};

mod algorithm;

#[test]
pub fn test() {
    let algorithm = MyAlgorithm {};
    let markets = HashMap::new();
    let exchange: Exchange = Exchange::new(markets);
    let mut exchanges: HashMap<String, Exchange> = HashMap::new();
    exchanges.insert("Binance".into(), exchange);
    let context = StockTrekContext::new(exchanges);
    let signal = algorithm.create_signal(context);
    println!("{:?}", signal);
}
