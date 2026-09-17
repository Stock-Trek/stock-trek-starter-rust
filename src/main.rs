use crate::{algorithms::cost_averaging::CostAveraging, market_data::MarketDataFactory};
use std::collections::HashMap;
use stock_trek::{
    EnqueueActionFn, ResolvedContext,
    errors::StockTrekResult,
    prelude::*,
    signals::{CexMarketDataByBaseContext, CexMarketDataByQuoteContext},
};

mod algorithms;
mod market_data;

pub fn main() -> StockTrekResult<()> {
    let algorithm = CostAveraging::default();

    let strategy_context = StrategyContext::new();
    let command = algorithm.strategy(&strategy_context);

    let bitcoin_tether_market = MarketDataFactory::random();
    let mut bitcoin_markets_by_quote = HashMap::new();
    bitcoin_markets_by_quote.insert(AssetId::TetherUSD, bitcoin_tether_market);
    let bitcoin_market = CexMarketDataByQuoteContext::new(bitcoin_markets_by_quote);
    let mut markets_by_base = HashMap::new();
    markets_by_base.insert(AssetId::Bitcoin, bitcoin_market);
    let binance_market_data = CexMarketDataByBaseContext::new(markets_by_base);

    let mut cex_market_data = HashMap::new();
    cex_market_data.insert(CexId::Binance, binance_market_data);
    let signal_context = SignalContext::new(cex_market_data);
    let signals = algorithm.signals(&signal_context);

    let mut actions = Vec::new();
    let enqueue_action: EnqueueActionFn = Box::new(move |action, policy| {
        actions.push((action.clone(), policy.clone()));
        Ok(())
    });
    let portfolios = HashMap::new();
    let portfolio = Portfolio::new(portfolios);
    let mut resolved_context = ResolvedContext {
        enqueue_action,
        portfolio,
        signals,
    };
    command.execute(&mut resolved_context)?;
    Ok(())
}
