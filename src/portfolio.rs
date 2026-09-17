use rand::RngExt;
use rust_decimal::Decimal;
use stock_trek::{
    Portfolio, PortfolioBuilder,
    types::{
        AccountId, Activation, AssetId, CexId, OrderRequest, Quantity, Side, Tag, TimeInForce,
    },
};

pub struct PortfolioFactory {
    portfolio: Portfolio,
}

pub const ACCOUNT_ID: &str = "random-portfolio";

impl From<PortfolioFactory> for Portfolio {
    fn from(value: PortfolioFactory) -> Self {
        value.portfolio
    }
}

impl PortfolioFactory {
    pub fn new(portfolio: Portfolio) -> Self {
        Self { portfolio }
    }

    pub fn random() -> Self {
        let mut rng = rand::rng();
        let cex = CexId::Binance;
        let account = AccountId::new(ACCOUNT_ID);
        let mut builder = PortfolioBuilder::new();
        let assets = [
            AssetId::Bitcoin,
            AssetId::Ethereum,
            AssetId::Solana,
            AssetId::TetherUSD,
            AssetId::USDCoin,
            AssetId::Dogecoin,
        ];
        for asset in assets {
            if rng.random_bool(0.5) {
                let count = Decimal::new(rng.random_range(1..1_000_000), 2);
                builder.asset_count(cex, account.clone(), asset, count);
            }
        }
        builder.asset_count(
            cex,
            account.clone(),
            AssetId::TetherUSD,
            Decimal::new(rng.random_range(1_000..100_000), 2),
        );
        builder.asset_count(
            cex,
            account.clone(),
            AssetId::Bitcoin,
            Decimal::new(rng.random_range(1..10), 2),
        );
        let tag = Tag::new("BuyBitcoinLimit");
        let order_request = OrderRequest::Limit {
            base: AssetId::Bitcoin,
            quote: AssetId::TetherUSD,
            side: Side::Buy,
            activation: Activation::Immediate,
            limit_price: Decimal::new(rng.random_range(1_000..100_000), 2),
            time_in_force: TimeInForce::GoodTillCancelled,
            quantity: Quantity::OfBase(Decimal::new(rng.random_range(1..10), 2)),
            tag: tag.clone(),
        };
        builder.pending_order(
            cex,
            account,
            tag,
            order_request,
            Decimal::ZERO,
            Decimal::ZERO,
        );
        Self::new(builder.build())
    }
}
