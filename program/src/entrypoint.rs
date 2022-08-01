use solana_program::{
  account_info::AccountInfo, entrypoint, entrypoint::ProgramResult, pubkey::Pubkey
};
use crate::processor::Processor;


// PROGRAM ENTRYPOINT
entrypoint!(process_instruction);

// PROGRAM FUNCTION
pub fn process_instruction(
  program_id: &Pubkey,
  accounts: &[AccountInfo],
  instruction_data: &[u8],
) -> ProgramResult {
  Processor::process(program_id, accounts, instruction_data)
}

// TEST MODULE
// Sanity Tests for initial checks pre-client
#[cfg(test)]
mod test {
  use super::*;
  use borsh::{BorshDeserialize, BorshSerialize};
  use solana_program::clock::Epoch;
  use std::mem;

  use crate::state::{User, Trade};
  use crate::instruction::{Make, Take};

  #[test]
  fn test_sanity() {

    // Program Id
    let program_id = Pubkey::default();

    // Account User
    let key = Pubkey::default();
    let owner = Pubkey::default();
    let mut lamports: u64 = 5000000000;
    let mut data = vec![0; User::LEN];
    let account = AccountInfo::new(
      &key, // Public key of account
      true, // is_signer
      true, // is_writable
      &mut lamports, // lamports
      &mut data, // data
      &owner, // Program owner
      false, // executable
      Epoch::default(), // epoch next owing rent
    );

    // Generate Program Derived Address (PDA)
    let slug = String::from("trade1");
    let (trade_pda, _trade_bump) = Pubkey::find_program_address(
      &[b"trade".as_ref(), slug.as_ref(), Pubkey::default().as_ref()],
      &program_id 
    );

    // Account Trade PDA
    let key_pda = trade_pda;
    let owner_pda = Pubkey::default();
    let mut lamports_pda: u64 = 0;
    let mut data_pda = vec![0; Trade::LEN];
    let account_pda = AccountInfo::new(
      &key_pda, // Public key of account
      true, // is_signer
      true, // is_writable
      &mut lamports_pda, // lamports
      &mut data_pda, // data
      &owner_pda, // Program owner
      false, // executable
      Epoch::default(), // epoch next owing rent
    );

    // Add to Accounts Vector - Authority, PDA, System Account
    let system_account = account.clone();
    let accounts = vec![account, account_pda, system_account];

    // Build Instruction 0 (Make)
    let ix_struct = Make {
      symbol: String::from("SOLUSD"),
      slug: slug,
      contract_size: 1, // 1 Sol
      direction: 0, // Long
      duration: 0, // 5Min
    };

    // // Build Instruction 1 (Take)
    // let ix_struct = Take {
    //   account_ref: Pubkey::default(),
    // };

    // Convert Instruction into Bytes
    let mut instruction = ix_struct.try_to_vec().unwrap();

    // Initialize instruction data with Routing
    let mut instruction_data: Vec<u8> = Vec::new();
    instruction_data.push(u8::to_le_bytes(0)[0]); // 0 for Make, 1 for Take

    // Add Maker Instruction to Instruction Data
    instruction_data.append(&mut instruction);

    // Send Request
    process_instruction(&program_id, &accounts, &instruction_data).unwrap();
  }
}