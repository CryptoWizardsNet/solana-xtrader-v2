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
  pub bump: u8, // Maker
  pub slug: String, // Maker
  pub symbol: String, // Maker
  pub contract_size: u8, // Maker
  pub direction: u8, // Maker
  pub duration: u8, // Maker
  pub unix_start: u32, // Set by Taker Call
  pub unix_end: u32, // Set by Taker Call (start + duration)
  pub benchmark_price: u32, // Set by Taker Call
  pub closing_price: u32, // Set by Claim Call
  pub order_status: u8, // 0 = Not Initialized, 1 = OpenOrder, 2 = InPlay, 3 = Claimed
}

// Get LEN of Trade Account
impl Trade {
  pub const LEN: usize = mem::size_of::<Pubkey>() * 2 + mem::size_of::<u32>() * 4 + mem::size_of::<u8>() * 5 + mem::size_of::<String>() * 2 + 8;
}
