use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{program_error::ProgramError, pubkey::Pubkey, msg};
use std::mem;

// Account 1 Struct
#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub struct Account1 {
  pub counter: u8,
}

// Account 1 Length Function
impl Account1 {
  pub fn len_space() -> Result<usize, ()> {
    Ok(mem::size_of::<u8>())
  }
}
