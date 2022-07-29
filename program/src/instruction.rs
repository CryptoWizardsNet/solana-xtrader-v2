use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{program_error::ProgramError, pubkey::Pubkey, msg};

// Struct for Instruction 1
#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub struct SomeDataStruct1 {
  pub some_data: u8,
}

// Struct for Instruction 2
#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub struct SomeDataStruct2 {
  pub some_more_data: u8,
}

#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub enum Instruction {
  Instruction1(SomeDataStruct1),
  Instruction2(SomeDataStruct2),
}

// Function to unpack instruction
impl Instruction {

  // Tag and Route request
  pub fn unpack(input: &[u8]) -> Result<Self, ProgramError> {
    let (tag, rest) = input.split_first().ok_or(ProgramError::InvalidInstructionData)?;
    msg!("Input: {:?}", input);
    msg!("Tag: {:?}", tag);
    msg!("Rest: {:?}", rest);

    Ok(match tag {
      0 => Self::Instruction1(SomeDataStruct1::try_from_slice(&rest)?),
      1 => Self::Instruction2(SomeDataStruct2::try_from_slice(&rest)?),
      _ => return Err(ProgramError::InvalidInstructionData.into()),
    })
  }
}
