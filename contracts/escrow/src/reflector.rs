/* reflector.rs */
use soroban_sdk::{contracttype, Address, Env, Symbol, Vec};

// Oracle contract interface exported as ReflectorClient
#[soroban_sdk::contractclient(name = "ReflectorClient")]
pub trait Contract {
    // Base oracle symbol the price is reported in
    fn base(e: Env) -> Asset;
    // All assets quoted by the contract
    fn assets(e: Env) -> Vec<Asset>;
    // Number of decimal places used to represent price for all assets quoted by the oracle
    fn decimals(e: Env) -> u32;
    // Quotes asset price in base asset at specific timestamp
    fn price(e: Env, asset: Asset, timestamp: u64) -> Option<PriceData>;
    // Quotes the most recent price for an asset
    fn lastprice(e: Env, asset: Asset) -> Option<PriceData>;
    // Quotes last N price records for the given asset
    fn prices(e: Env, asset: Asset, records: u32) -> Option<Vec<PriceData>>;
    // Quotes the most recent cross price record for the pair of assets
    fn x_last_price(e: Env, base_asset: Asset, quote_asset: Asset) -> Option<PriceData>;
    // Quotes the cross price for the pair of assets at specific timestamp
    fn x_price(e: Env, base_asset: Asset, quote_asset: Asset, timestamp: u64) -> Option<PriceData>;
    // Quotes last N cross price records of for the pair of assets
    fn x_prices(e: Env, base_asset: Asset, quote_asset: Asset, records: u32) -> Option<Vec<PriceData>>;
    // Quotes the time-weighted average price for the given asset over N recent records
    fn twap(e: Env, asset: Asset, records: u32) -> Option<i128>;
    // Quotes the time-weighted average cross price for the given asset pair over N recent records
    fn x_twap(e: Env, base_asset: Asset, quote_asset: Asset, records: u32) -> Option<i128>;
    // Price feed resolution (default tick period timeframe, in seconds - 5 minutes by default)
    fn resolution(e: Env) -> u32;
    // Historical records retention period, in seconds (24 hours by default)
    fn period(e: Env) -> Option<u64>;
    // The most recent price update timestamp
    fn last_timestamp(e: Env) -> u64;
    // Contract protocol version
    fn version(e: Env) -> u32;
    // Contract admin address
    fn admin(e: Env) -> Option<Address>;
    // Note: it's safe to remove any methods not used by the consumer contract from this client trait
}

// Quoted asset definition
#[contracttype(export = false)]
#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub enum Asset {
    Stellar(Address), // for Stellar Classic and Soroban assets
    Other(Symbol)     // for any external currencies/tokens/assets/symbols
}

// Price record definition
#[contracttype(export = false)]
#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct PriceData {
    pub price: i128,   // asset price at given point in time
    pub timestamp: u64 // record timestamp
}

// Possible runtime errors
#[soroban_sdk::contracterror(export = false)]
#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub enum Error {
    AlreadyInitialized = 0,
    Unauthorized = 1,
    AssetMissing = 2,
    AssetAlreadyExists = 3,
    InvalidConfigVersion = 4,
    InvalidTimestamp = 5,
    InvalidUpdateLength = 6,
    AssetLimitExceeded = 7
}