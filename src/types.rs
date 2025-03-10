use anchor_lang::prelude::*;
use fixed::types::I80F48;

/// Multiplied by 1_000, to save compute units.
pub const SPOT_INITIAL_MARGIN_REQ: u64 = 1_100_000;

/// Multiplied by 1_000, to save compute units.
pub const SPOT_MAINT_MARGIN_REQ: u64 = 1_030_000;

/// In microUSD.
pub const DUST_THRESHOLD: i64 = 1_000_000;

pub const MAX_COLLATERALS: usize = 25;
pub const MAX_MARKETS: usize = 50;
pub const MAX_ORACLE_SOURCES: usize = 3;

#[derive(
    AnchorDeserialize,
    AnchorSerialize,
    Copy,
    Clone,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
)]
pub struct Symbol {
    data: [u8; 24],
}

impl Symbol {
    pub fn is_nil(&self) -> bool {
        self.data.iter().all(|x| *x == 0)
    }
}

impl<'a> TryFrom<&'a str> for Symbol {
    type Error = ParseSymbolError;
    fn try_from(x: &'a str) -> std::result::Result<Self, Self::Error> {
        if x.len() > 24 {
            Err(Self::Error {})
        } else {
            let mut data = [0u8; 24];
            data[0..x.len()].copy_from_slice(x.as_bytes());
            Ok(Self { data })
        }
    }
}

#[derive(Debug)]
pub struct ParseSymbolError {}

impl TryFrom<&String> for Symbol {
    type Error = ParseSymbolError;
    fn try_from(x: &String) -> std::result::Result<Self, Self::Error> {
        Symbol::try_from(x.as_str())
    }
}

impl TryFrom<String> for Symbol {
    type Error = ParseSymbolError;
    fn try_from(x: String) -> std::result::Result<Self, Self::Error> {
        Symbol::try_from(x.as_str())
    }
}

impl From<Symbol> for String {
    fn from(sym: Symbol) -> String {
        String::from(&sym)
    }
}

impl From<&Symbol> for String {
    fn from(sym: &Symbol) -> String {
        let mut end = 0;
        while end < sym.data.len() && sym.data[end] != 0 {
            end += 1;
        }
        String::from_utf8_lossy(&sym.data[0..end]).into()
    }
}

#[derive(PartialEq, Copy, Clone)]
pub enum FractionType {
    Maintenance,
    Initial,
    Cancel,
}

#[derive(AnchorSerialize, AnchorDeserialize, Copy, Clone, PartialEq)]
pub enum OrderType {
    Limit = 0,
    ImmediateOrCancel = 1,
    PostOnly = 2,
    ReduceOnlyIoc = 3,
    ReduceOnlyLimit = 4,
    FillOrKill = 5,
}

#[derive(
    AnchorDeserialize,
    AnchorSerialize,
    Copy,
    Clone,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
)]
pub struct WrappedI80F48 {
    pub data: i128,
}

impl WrappedI80F48 {
    pub fn zero() -> Self {
        Self::from(I80F48::ZERO)
    }
}

impl From<I80F48> for WrappedI80F48 {
    fn from(i: I80F48) -> Self {
        Self { data: i.to_bits() }
    }
}

impl From<WrappedI80F48> for I80F48 {
    fn from(i: WrappedI80F48) -> Self {
        Self::from_bits(i.data)
    }
}

#[zero_copy]
#[repr(packed)]
pub struct CollateralInfo {
    pub mint: Pubkey,
    pub oracle_symbol: Symbol,
    pub decimals: u8,
    pub weight: u16, //  in permil
    pub liq_fee: u16, // in permil

    // borrow lending info
    pub is_borrowable: bool,
    pub optimal_util: u16, // in permil
    pub optimal_rate: u16, // in permil
    pub max_rate: u16, // in permil
    pub og_fee: u16, // in bps

    // swap info
    pub is_swappable: bool,
    pub serum_open_orders: Pubkey,

    pub max_deposit: u64,    // in smol
    pub dust_threshold: u16, // in smol

    _padding: [u8; 384],
}

impl CollateralInfo {
    pub fn is_empty(&self) -> bool {
        self.mint == Pubkey::default()
    }
}

#[zero_copy]
#[repr(packed)]
pub struct PerpMarketInfo {
    // info
    pub symbol: Symbol, // Convention ex: "BTC-EVER-C" or "BTC-PERP"
    pub oracle_symbol: Symbol,
    pub perp_type: PerpType,
    // settings
    pub asset_decimals: u8,
    pub asset_lot_size: u64,
    pub quote_lot_size: u64,
    pub strike: u64, // in smolUSD per bigAsset
    pub base_imf: u16, // in permil (i.e. 1% <=> 10 permil)
    pub liq_fee: u16, // in permil
    // zoDex dex keys
    pub dex_market: Pubkey,

    _padding: [u8; 320],
}

#[derive(Copy, Clone, PartialEq)]
pub enum PerpType {
    Future = 0,
    CallOption = 1,
    PutOption = 2,
    Square = 3,
}

#[zero_copy]
#[repr(packed)]
pub struct OpenOrdersInfo {
    pub key: Pubkey,
    pub native_pc_total: i64,
    pub pos_size: i64,
    pub realized_pnl: i64,
    pub coin_on_bids: u64,
    pub coin_on_asks: u64,
    pub order_count: u8,
    pub funding_index: i128,
}

#[zero_copy]
#[repr(packed)]
pub struct OracleCache {
    pub symbol: Symbol,
    pub sources: [OracleSource; MAX_ORACLE_SOURCES],
    pub last_updated: u64,
    pub price: WrappedI80F48, // smol quote per smol asset
    pub twap: WrappedI80F48,
    pub base_decimals: u8, // actual decimal of the mint
    pub quote_decimals: u8,
}

#[derive(Copy, Clone)]
pub enum OracleType {
    Nil = 0,
    Pyth,
    Switchboard,
}

#[zero_copy]
#[repr(packed)]
pub struct OracleSource {
    pub ty: OracleType,
    pub key: Pubkey,
}

#[zero_copy]
#[repr(packed)]
pub struct MarkCache {
    pub price: WrappedI80F48, // smol usd per smol asset
    /// Hourly twap sampled every 5min.
    pub twap: TwapInfo,
}

#[zero_copy]
#[repr(packed)]
pub struct TwapInfo {
    pub cumul_avg: WrappedI80F48,
    pub open: WrappedI80F48,
    pub high: WrappedI80F48,
    pub low: WrappedI80F48,
    pub close: WrappedI80F48,
    pub last_sample_start_time: u64,
}

#[zero_copy]
#[repr(packed)]
pub struct BorrowCache {
    pub supply: WrappedI80F48, // in smol
    pub borrows: WrappedI80F48, // in smol
    pub supply_multiplier: WrappedI80F48, // earned interest per asset supplied
    pub borrow_multiplier: WrappedI80F48, // earned interest per asset borrowed
    pub last_updated: u64,
}

#[account(zero_copy)]
#[repr(packed)]
pub struct State {
    pub signer_nonce: u8,
    pub admin: Pubkey,
    pub cache: Pubkey,
    pub swap_fee_vault: Pubkey,
    pub insurance: u64, // in smol usd

    /// Fees accrued through borrow lending
    pub fees_accrued: [u64; MAX_COLLATERALS], // in smol usd
    pub vaults: [Pubkey; MAX_COLLATERALS],
    pub collaterals: [CollateralInfo; MAX_COLLATERALS],
    pub perp_markets: [PerpMarketInfo; MAX_MARKETS],

    pub total_collaterals: u16,
    pub total_markets: u16,

    _padding: [u8; 1280],
}

#[account(zero_copy)]
#[repr(packed)]
pub struct Margin {
    pub nonce: u8,
    pub authority: Pubkey,
    /// Mapped to the state collaterals array, divided by entry sup or bor_index
    pub collateral: [WrappedI80F48; MAX_COLLATERALS],
    pub control: Pubkey,

    _padding: [u8; 320],
}

#[account(zero_copy)]
#[repr(packed)]
pub struct Cache {
    pub oracles: [OracleCache; MAX_COLLATERALS],
    /// Mapped to `State.perp_markets`
    pub marks: [MarkCache; MAX_MARKETS],
    pub funding_cache: [i128; MAX_MARKETS], // long to short
    /// Mapped to 'State.collaterals'
    pub borrow_cache: [BorrowCache; MAX_COLLATERALS],
}

#[account(zero_copy)]
#[repr(packed)]
pub struct Control {
    pub authority: Pubkey,
    /// Mapped to `State.perp_markets`
    pub open_orders_agg: [OpenOrdersInfo; MAX_MARKETS],
}

#[derive(Copy, Clone)]
pub enum FeeTier {
    Base,
    ZO2,
    ZO3,
    ZO4,
    ZO5,
    ZO6,
    MSRM,
}
