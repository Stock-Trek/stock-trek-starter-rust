use chrono::{DateTime, Utc};
use rand::{RngExt, rngs::ThreadRng};
use std::{
    collections::HashMap,
    time::{SystemTime, UNIX_EPOCH},
};
use stock_trek::{
    markets::{
        AlignedWindow, Market, MarketAlignedWindow, MarketCandle, MarketOhlcv, MarketOrderBook,
        MarketQuote, MarketRollingWindow, MarketTick, MarketTicks, RollingWindow,
    },
    signals::{CexMarketDataByBaseContext, CexMarketDataByQuoteContext, SignalContext},
    types::{AssetId, CexId},
};
use strum::IntoEnumIterator;

pub struct MarketDataFactory {
    exchange: CexId,
    base: AssetId,
    quote: AssetId,
    market: Market,
}

impl From<MarketDataFactory> for SignalContext {
    fn from(value: MarketDataFactory) -> Self {
        let MarketDataFactory {
            exchange,
            base,
            quote,
            market,
        } = value;
        let markets_by_quote = HashMap::from([(quote, market)]);
        let base_market = CexMarketDataByQuoteContext::new(markets_by_quote);
        let markets_by_base = HashMap::from([(base, base_market)]);
        let market_data = CexMarketDataByBaseContext::new(markets_by_base);
        let cex_market_data = HashMap::from([(exchange, market_data)]);
        SignalContext::new(cex_market_data)
    }
}

impl MarketDataFactory {
    pub fn new(exchange: CexId, base: AssetId, quote: AssetId, market: Market) -> Self {
        Self {
            exchange,
            base,
            quote,
            market,
        }
    }

    pub fn random() -> Self {
        let mut rng = rand::rng();
        Self::new(
            CexId::Binance,
            AssetId::Bitcoin,
            AssetId::TetherUSD,
            Self::market(&mut rng),
        )
    }

    fn market(rng: &mut ThreadRng) -> Market {
        let price = rng.random_range(1.0..100_000.0);
        Market {
            base_increment: Self::increment(rng),
            quote_increment: Self::increment(rng),
            minimum_notional: rng.random_range(1.0..100.0),
            ticks: Self::ticks(price, rng),
            rolling: Self::rolling(price, rng),
            aligned: Self::aligned(price, rng),
            order_book: Self::order_book(price, rng),
        }
    }

    fn increment(rng: &mut ThreadRng) -> f64 {
        10f64.powi(-rng.random_range(1..=6))
    }

    fn ticks(price: f64, rng: &mut ThreadRng) -> MarketTicks {
        let count = rng.random_range(1..=20usize);
        let now_millis = Self::now_millis();
        let mut ticks = Vec::with_capacity(count);
        let mut last_price = price;
        for index in 0..count {
            let spread = last_price * rng.random_range(0.0001..0.001);
            let tick = MarketTick {
                timestamp_millis: now_millis.saturating_sub((count - index) as u64 * 1_000),
                bid: Self::quote(last_price - spread, rng),
                ask: Self::quote(last_price + spread, rng),
                last: Self::quote(last_price, rng),
            };
            last_price *= 1.0 + rng.random_range(-0.01..0.01);
            ticks.push(tick);
        }
        MarketTicks::new(ticks)
    }

    fn rolling(price: f64, rng: &mut ThreadRng) -> MarketRollingWindow {
        let end = Self::now();
        let mut candles = HashMap::new();
        for window in RollingWindow::iter() {
            let start = window
                .checked_sub(end)
                .expect("window start is out of range");
            candles.insert(window, Self::candle(rng, start, end, false, price));
        }
        MarketRollingWindow::new(candles)
    }

    fn aligned(price: f64, rng: &mut ThreadRng) -> MarketAlignedWindow {
        let end = Self::now();
        let mut candles = HashMap::new();
        for window in AlignedWindow::iter() {
            let start = window
                .checked_sub(end)
                .expect("window start is out of range");
            candles.insert(
                window,
                Self::aligned_candles(rng, window, start, end, price),
            );
        }
        MarketAlignedWindow::new(candles)
    }

    fn aligned_candles(
        rng: &mut ThreadRng,
        window: AlignedWindow,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
        price: f64,
    ) -> Vec<MarketCandle> {
        assert!(end > start);
        let mut candles = Vec::new();
        let mut window_start = start;
        let mut open = price;
        while window_start < end {
            let Some(window_end) = window.checked_add(window_start) else {
                break;
            };
            let is_candle_closed = window_end <= end;
            let candle = Self::candle(rng, window_start, window_end, is_candle_closed, open);
            open = candle.ohlcv.close;
            candles.push(candle);
            window_start = window_end;
        }
        candles
    }

    fn candle(
        rng: &mut ThreadRng,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
        is_candle_closed: bool,
        open: f64,
    ) -> MarketCandle {
        let start_time_millis_inc = start.timestamp_millis() as u64;
        let end_time_millis_exc = end.timestamp_millis() as u64;
        let duration_millis = end_time_millis_exc.saturating_sub(start_time_millis_inc);
        let ohlcv = Self::ohlcv(open, rng);
        let trade_count = rng.random_range(0..10_000);
        MarketCandle {
            start_time_millis_inc,
            end_time_millis_exc,
            duration_millis,
            is_candle_closed,
            ohlcv,
            trade_count,
        }
    }

    fn ohlcv(open: f64, rng: &mut ThreadRng) -> MarketOhlcv {
        let close = open * (1.0 + rng.random_range(-0.05..0.05));
        let high = open.max(close) * (1.0 + rng.random_range(0.0..0.05));
        let low = open.min(close) * (1.0 - rng.random_range(0.0..0.05));
        let volume = rng.random_range(0.0..10_000.0);
        let quote_volume = volume * open;
        let vwap = rng.random_range(low..=high);
        MarketOhlcv::new(open, high, low, close, volume, quote_volume, vwap)
    }

    fn order_book(price: f64, rng: &mut ThreadRng) -> MarketOrderBook {
        let bids = (0..rng.random_range(1..=10u32))
            .map(|level| Self::quote(price * (1.0 - 0.001 * f64::from(level + 1)), rng))
            .collect();
        let asks = (0..rng.random_range(1..=10u32))
            .map(|level| Self::quote(price * (1.0 + 0.001 * f64::from(level + 1)), rng))
            .collect();
        MarketOrderBook::new(bids, asks)
    }

    fn quote(price: f64, rng: &mut ThreadRng) -> MarketQuote {
        MarketQuote {
            price,
            quantity: rng.random_range(0.1..100.0),
        }
    }

    fn now() -> DateTime<Utc> {
        let timestamp_nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("SystemTime is negative")
            .as_nanos() as i64;
        DateTime::from_timestamp_nanos(timestamp_nanos)
    }

    fn now_millis() -> u64 {
        Self::now().timestamp_millis() as u64
    }
}
