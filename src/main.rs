use crate::{
    algorithms::cost_averaging::CostAveraging,
    market_data::MarketDataFactory,
    portfolio::{ACCOUNT_ID, PortfolioFactory},
};
use stock_trek::{EnqueueActionFn, ResolvedContext, errors::StockTrekResult, prelude::*};

mod algorithms;
mod market_data;
mod portfolio;

pub fn main() -> StockTrekResult<()> {
    let algorithm = CostAveraging::default();

    let strategy_context = StrategyContext::new();
    let command = algorithm.strategy(&strategy_context);

    let signal_context: SignalContext = MarketDataFactory::random().into();
    let mut signals = algorithm.signals(&signal_context);
    signals.write(
        &SignalKey::new_required("ACCOUNT"),
        AccountId::new(ACCOUNT_ID),
    );

    let mut actions = Vec::new();
    let enqueue_action: EnqueueActionFn = Box::new(move |action, policy| {
        actions.push((action.clone(), policy.clone()));
        Ok(())
    });
    let portfolio: Portfolio = PortfolioFactory::random().into();
    let mut resolved_context = ResolvedContext {
        enqueue_action,
        portfolio,
        signals,
    };
    command.execute(&mut resolved_context)?;
    Ok(())
}
