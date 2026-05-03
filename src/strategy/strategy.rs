use std::cmp::Ordering;
use stock_trek::prelude::*;

pub struct CostAveraging {
    pub key_market_exists: ScratchKey<bool>,
    pub key_satoshi_price: ScratchKey<f64>,
}

impl Default for CostAveraging {
    fn default() -> Self {
        Self {
            key_market_exists: ScratchKey::new_optional("MARKET_EXISTS", false),
            key_satoshi_price: ScratchKey::new_required("SATOSHI_PRICE"),
        }
    }
}

#[register_strategy(default)]
impl Strategy for CostAveraging {
    fn market_calculations(&self, context: StrategyContext) -> StockTrekResult<ScratchPad> {
        let mut scratch_pad = ScratchPad::new();
        if let Some(binance) = context.exchanges.get(&ExchangeId::Binance) {
            let btc_usdt = context.symbol("BTC", "USDT");
            let market_opt = binance.market_for(&btc_usdt)?;
            if let Some(market) = market_opt {
                scratch_pad.write(&self.key_market_exists, true);
                let satoshi_price = market.ticks.ticks[0].last.price / 1_000_000.0;
                scratch_pad.write(&self.key_satoshi_price, satoshi_price);
            }
        }
        Ok(scratch_pad)
    }
    fn action_resolver(&self, context: ResolverContext) -> StockTrekResult<Resolver> {
        Ok(context.resolvers.if_else(
            context.predicates.scratch_pad(&self.key_market_exists),
            context.resolvers.if_else(
                context.predicates.compare(
                    context.portfolio.asset_in_exchange(
                        context.literals.exchange(ExchangeId::Binance),
                        context.literals.asset("USDT"),
                    ),
                    Ordering::Greater,
                    context.scratch_pad.number(&self.key_satoshi_price),
                ),
                context.resolvers.action(context.actions.order_request(
                    ExchangeId::Binance,
                    OrderRequest {
                        account_type: AccountType::Spot,
                        client_order_id: None,
                        order_type: OrderType::Market,
                        quantity: 1.0 / 1_000_000.0,
                        reduce_only: false,
                        side: OrderSide::Buy,
                        symbol: Symbol::new("BTC", "USDT"),
                        time_in_force: TimeInForce::Fok,
                    },
                )),
                context.resolvers.no_op(),
            ),
            context.resolvers.no_op(),
        ))
    }
}
