use solana_program::{
  program_error::ProgramError,
};
use thiserror::Error;

// Custom Error Enum
#[derive(Error, Debug, Copy, Clone)]
pub enum TradeError {
    #[error("Invalid Instruction")]
    InvalidInstruction,

    #[error("Invalid User Account")]
    InvalidUserAccount,

    #[error("Invalid Trade Account")]
    InvalidTradeAccount,

    #[error("Invalid Trade Data")]
    InvalidTradeData,

    #[error("Invalid Contract Size")]
    InvalidContractSize,

    #[error("Duration Error")]
    InvalidDurationCalculation,

    #[error("A Matched Trade Already Exists for This Account")]
    AlreadyExistingTrade,

    #[error("Not Enough SOL (Lamports)")]
    NotEnoughLamports,

    #[error("Account not Writable")]
    AccountNotWritable,
}

// Custom Error Function
impl From<TradeError> for ProgramError {
    fn from(e: TradeError) -> Self {
        return ProgramError::Custom(e as u32);
    }
}