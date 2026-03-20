use std::{fmt, path::PathBuf, time::SystemTime};
use clap::Parser;
use serde::{Serialize, Deserialize};

use crate::error::QuoutesError;

const BASE_VOLUME: u32 = 1000;
const MAX_VOLUME: f64 = 5000.0;
const BASE_PRICE: f64 = 1.0;
const MAX_PRICE: f64 = 10.0;

const SUBSCRIBE_FILE_PATH: &str = "c:/tmp/ss.txt";

pub const SERVER_HOST: &str = "127.0.0.1";
pub const SERVER_PORT: &str = "7878";
pub const SERVER_UDP_PORT: &str = "7879";

pub const UDP_HOST: &str = "127.0.0.1";
pub const UDP_PORT: u16 = 34254;

pub const BASE_COMMAND_PREFIX: &str = "STREAM";
pub const SUCCESS_SERVER_RESPONSE: &str = "Connection ok";
pub const PING_MESSAGE: &str = "Ping";

pub const GENERATE_QUOTES_PERIOD: u64 = 10;
pub const CLIENT_QUOTES_PERIOD: u64 = 10;

pub const PING_PERIOD: u64 = 2;
pub const PING_TIMEOUT_SEC: u64 = 5;

pub const CLIENT_TIME_OUT_MS: u64 = 1000;

pub mod server;
pub mod broadcaster;
pub mod quotes_generator;
pub mod error;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct StockQuote {
    pub ticker: String,
    pub price: f64,
    pub volume: u32,
    pub timestamp: u64,
}

impl Default for StockQuote {
    fn default() -> Self {
        Self::new()
    }
}

impl StockQuote {
    pub fn new() -> Self {
        Self { 
            ticker: String::new(), 
            price: 0.0, 
            volume: 0, 
            timestamp: 0
        }
    }
    
    pub fn to_string(&self) -> String {
        format!("{}|{}|{}|{}", self.ticker, self.price, self.volume, self.timestamp)
    }
    
    pub fn from_string(s: &str) -> Option<Self> {
        let parts: Vec<&str> = s.split('|').collect();
        if parts.len() == 4 {
            Some(StockQuote {
                ticker: parts[0].to_string(),
                price: parts[1].parse().ok()?,
                volume: parts[2].parse().ok()?,
                timestamp: parts[3].parse().ok()?,
            })
        } else {
            None
        }
    }

    pub fn generate_indicators(&mut self, ticker: &str ) -> Result<(), QuoutesError> {
        let volume = BASE_VOLUME + (rand::random::<f64>() * MAX_VOLUME) as u32;
        let price = BASE_PRICE + (rand::random::<f64>() * MAX_PRICE) as f64;
        self.ticker = ticker.to_string();
        self.price = price;
        self.volume = volume;
        self.timestamp = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH)?.as_millis() as u64;
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StockQuoteSet {
    pub tickers: Vec<StockQuote>,
}

impl Default for StockQuoteSet {
    fn default() -> Self {
        Self::new()
    }
}

impl StockQuoteSet {
    pub fn new() -> Self {
        Self { 
            tickers: Vec::new(), 
        }
    }

    pub fn get_size(&self) -> usize {
        self.tickers.len()
    }

    pub fn push(&mut self, item: StockQuote) {
        self.tickers.push(item)
    }

    pub fn set_quotes(&mut self, result: Vec<StockQuote>) {
        self.tickers = result;
    }

    pub fn get_quotes(self) -> Vec<StockQuote> {
        self.tickers
    }

    pub fn filter(self, tickers_to_filter: Vec<String>) -> StockQuoteSet {
        let mut result = StockQuoteSet::new();
        let quotes: Vec<StockQuote> = 
            self.tickers.into_iter().filter
                (|p| tickers_to_filter.contains(&p.ticker) || tickers_to_filter.contains(&"*".to_string())).collect();
        result.set_quotes(quotes);
        result
    }
}

impl fmt::Display for StockQuoteSet {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut result = String::new();
        for item in &self.tickers {
            result = result.to_owned() + item.to_string().as_str() + "\n";
        }
        write!(f, "{}", result)
    }
}

#[derive(Parser, Debug)]
#[command(name = "quote_client")]
pub struct ClientArgs {
    #[arg(long, default_value = SERVER_HOST)]
    pub tcp_host: String,

    #[arg(long, default_value = SERVER_PORT)]
    pub tcp_port: String,

    #[arg(long, default_value = UDP_HOST)]
    pub udp_host: String,

    #[arg(long, default_value_t = UDP_PORT)]
    pub udp_port: u16,

    #[arg(long, default_value = SUBSCRIBE_FILE_PATH)]
    pub subscribe_file: PathBuf
}

