use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{pubkey::Pubkey};
use std::mem;


// User Account
#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
pub struct User {
  pub trades_placed: u32,
}

// Get LEN of User Account
impl User {
  pub const LEN: usize = mem::size_of::<u32>();
}

// Trade Account
#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
pub struct Trade {
  pub maker: Pubkey, // Maker
  pub taker: Pubkey, // Set by Taker Call
  pub trade_account: Pubkey, // Maker
  pub bump: u8, // Maker
  pub slug: String, // Maker
  pub symbol: String, // Maker
  pub contract_size: u8, // Maker
  pub direction: u8, // Maker
  pub duration: u8, // Maker
  pub unix_created: u32, // Maker
  pub unix_start: u32, // Taker
  pub unix_end: u32, // Taker (start + duration)
  pub benchmark_price: i128, // Taker
  pub closing_price: i128, // Claimer
  pub order_status: u8, // All Instructions: 0 = Not Initialized, 1 = OpenOrder, 2 = InPlay, 3 = Claimed
}

// Get LEN of Trade Account
impl Trade {
  pub const LEN: usize = mem::size_of::<Pubkey>() * 3 + mem::size_of::<u32>() * 2 + mem::size_of::<i128>() * 2 + 
  mem::size_of::<u8>() * 5 + mem::size_of::<String>() * 2 + 8; // Add 4 Bytes per String
}
