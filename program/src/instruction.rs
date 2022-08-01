use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{program_error::ProgramError, pubkey::Pubkey, msg};
use crate::error::TradeError;


// MAKE
// Make Instruction
#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub struct Make {
  pub symbol: String,
  pub slug: String, // For unique Trade Account creation
  pub contract_size: u8, // 0 = 0.1 Sol, 1 = 1 Sol and 2 = 5 Sol
  pub direction: u8, // 0 = Long, 1 = Short
  pub duration: u8, // 0 = 5Min, 1 = 1 Hour, 2 = 1 Day
}

// TAKE
// Take Instruction
#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub struct Take {
  pub direction: u8,
}

// Trade Instruction
#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub enum TradeInstruction {
  MakeTrade(Make),
  TakeTrade(Take),
  Claim,
}

// Unpack Instruction
impl TradeInstruction {
  pub fn unpack(input: &[u8]) -> Result<Self, ProgramError> {
    let (tag, rest) = input.split_first().ok_or(TradeError::InvalidInstruction)?;
    msg!("Tag: {:?}", tag);
    msg!("Expected Length: {:?}", rest.len());

    Ok(match tag {
      0 => {
        let payload = Make::try_from_slice(rest).unwrap();
        msg!("Payload: {:?}", payload);
        Self::MakeTrade ( Make {
            symbol: payload.symbol,
            slug: payload.slug,
            contract_size: payload.contract_size,
            direction: payload.direction,
            duration: payload.duration,
          }
        )
      },
      1 => {
        let payload = Take::try_from_slice(rest).unwrap();
        msg!("Payload: {:?}", payload);
        Self::TakeTrade ( Take {
          direction: payload.direction,
          }
        )
      },
      2 => Self::Claim,
      _ => return Err(TradeError::InvalidInstruction.into()),
    })
  }
}
