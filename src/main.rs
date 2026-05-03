use crate::strategy::strategy::CostAveraging;
use std::collections::HashMap;
use stock_trek::prelude::*;

mod strategy;

pub fn main() -> StockTrekResult<()> {
    let strategy = CostAveraging::default();
    let exchange = ExchangeFactory::stub();
    let mut exchanges = HashMap::new();
    exchanges.insert(ExchangeId::Binance, exchange);
    let resolver_context: ResolverContext = ResolverContext::new();
    let strategy_context: StrategyContext = StrategyContext::new(exchanges);
    let resolver = strategy.action_resolver(resolver_context)?;
    let scratch_pad = strategy.market_calculations(strategy_context)?;
    println!("{:?}", scratch_pad);
    let portfolio = PortfolioFactory::stub();
    let resolved_context = ResolvedContext {
        portfolio,
        scratch_pad,
    };
    let mut actions = Vec::new();
    println!("resolve");
    resolver.resolve(&resolved_context, &mut actions)?;
    Ok(())
}
