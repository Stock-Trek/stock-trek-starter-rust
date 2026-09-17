use chrono::{DateTime, Utc};
use rand::{RngExt, rngs::ThreadRng};
use std::{
    collections::HashMap,
    time::{SystemTime, UNIX_EPOCH},
};
use stock_trek::{
    markets::{
        AlignedWindow, Market, MarketAlignedWindow, MarketCandle, MarketOhlcv, MarketOrderBook,
        MarketQuote, MarketRollingWindow, MarketTicks, RollingWindow,
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
    pub fn random() -> Market {
        let ticks = MarketTicks::new(vec![]);
        let rolling = Self::rolling();
        let aligned = Self::aligned();
        Market {
            base_increment: 0.01,
            quote_increment: 0.01,
            minimum_notional: 1.0,
            ticks,
            rolling,
            aligned,
            order_book: Self::order_book(),
        }
    }
    fn aligned() -> MarketAlignedWindow {
        let timestamp_nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("SystemTime is negative")
            .as_nanos() as i64;
        let end = DateTime::from_timestamp_nanos(timestamp_nanos);
        let mut candles = HashMap::new();
        for window in AlignedWindow::iter() {
            let start = window.checked_sub(end).unwrap();
            candles.insert(window, Self::aligned_candles(window, start, end));
        }
        MarketAlignedWindow::new(candles)
    }
    fn rolling() -> MarketRollingWindow {
        let rng = rand::rng();
        let timestamp_nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("SystemTime is negative")
            .as_nanos() as i64;
        let end = DateTime::from_timestamp_nanos(timestamp_nanos);
        let mut candles = HashMap::new();
        for window in RollingWindow::iter() {
            let start = window.checked_sub(end).unwrap();
            let is_candle_closed = false;
            let open = 120.0;
            candles.insert(
                window,
                Self::candle(rng.clone(), start, end, is_candle_closed, open),
            );
        }
        MarketRollingWindow::new(candles)
    }
    fn aligned_candles(
        window: AlignedWindow,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> Vec<MarketCandle> {
        assert!(end > start);
        let mut rng = rand::rng();
        let mut candles = Vec::new();
        let mut window_start = start;
        let mut open = 100.0;
        loop {
            if let Some(window_end) = window.checked_add(window_start) {
                let is_candle_closed = window_end <= end;
                let start_time_millis_inc = window_start.timestamp_millis() as u64;
                let end_time_millis_exc = window_end.timestamp_millis() as u64;
                let duration_millis = end_time_millis_exc - start_time_millis_inc;
                let ohlcv = Self::ohlcv(open, &mut rng);
                open = ohlcv.close;
                let trade_count = rng.random_range(0..10_000);
                candles.push(MarketCandle {
                    start_time_millis_inc,
                    end_time_millis_exc,
                    duration_millis,
                    is_candle_closed,
                    ohlcv,
                    trade_count,
                });
                window_start = window_end;
            } else {
                break;
            }
        }
        candles
    }
    fn candle(
        mut rng: ThreadRng,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
        is_candle_closed: bool,
        open: f64,
    ) -> MarketCandle {
        let start_time_millis_inc = start.timestamp_millis() as u64;
        let end_time_millis_exc = end.timestamp_millis() as u64;
        let duration_millis = end_time_millis_exc - start_time_millis_inc;
        let ohlcv = Self::ohlcv(open, &mut rng);
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
        let high = open + rng.random_range(1.0..2.0);
        let low = open + rng.random_range(-2.0..-1.0);
        let close = open + rng.random_range(-1.0..1.0);
        let volume = rng.random_range(0.0..10_000.0);
        let quote_volume = rng.random_range(0.0..10_000.0);
        let vwap = open + rng.random_range(-1.0..1.0);
        MarketOhlcv::new(open, high, low, close, volume, quote_volume, vwap)
    }
    fn order_book() -> MarketOrderBook {
        MarketOrderBook::new(Self::order_book_bids(), Self::order_book_asks())
    }
    fn order_book_bids() -> Vec<MarketQuote> {
        vec![Self::market_quote()]
    }
    fn order_book_asks() -> Vec<MarketQuote> {
        vec![Self::market_quote()]
    }
    fn market_quote() -> MarketQuote {
        MarketQuote {
            price: 123.0,
            quantity: 1.0,
        }
    }
}
