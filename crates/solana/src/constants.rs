use solana_sdk::pubkey;
use solana_sdk::pubkey::Pubkey;

pub const SOL_BASE_PATH: &str = "https://api.mainnet-beta.solana.com";

pub const JUP_BASE_PATH: &str = "https://api.jup.ag/swap/v1";
pub const USDC_MINT: Pubkey = pubkey!("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v");
pub const NATIVE_MINT: Pubkey = pubkey!("So11111111111111111111111111111111111111112");

pub const DEFAULT_SLIPPAGE_BPS: u16 = 2000; // 20%

pub const HASH_EXPIRATION: std::time::Duration = std::time::Duration::from_secs(15);
