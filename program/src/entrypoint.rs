use solana_program::{
  account_info::AccountInfo, entrypoint, entrypoint::ProgramResult, pubkey::Pubkey, msg
};
use crate::processor::Processor;


// ENTRYPOINT
// Signal for BPF Loader to access program
entrypoint!(process_instruction);

// Program
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
  use crate::instruction::{SomeDataStruct1};
  use solana_program::clock::Epoch;
  use std::mem;

  #[test]
  fn test_sanity() {

    // Program Id
    let program_id = Pubkey::default();

    // Account
    let key = Pubkey::default();
    let owner = Pubkey::default();
    let mut lamports: u64 = 0;
    let space = mem::size_of::<u8>();
    let mut data = vec![0; space];
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

    // Add to Accounts Vector
    let accounts = vec![account];

    // Build Instruction
    let ix_struct = SomeDataStruct1 {
      some_data: 0,
    };

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
