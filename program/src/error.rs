use solana_program::{
  program_error::ProgramError,
};
use thiserror::Error;

// Custom Error Enum
#[derive(Error, Debug, Copy, Clone)]
pub enum BlogError {
    #[error("Invalid Instruction")]
    InvalidInstruction,

    #[error("Invalid Blog Account")]
    InvalidBlogAccount,

    #[error("Invalid Post Account")]
    InvalidPostAccount,

    #[error("Invalid Post Data")]
    InvalidPostData,

    #[error("Account not Writable")]
    AccountNotWritable,
}

// Custom Error Function
impl From<BlogError> for ProgramError {
    fn from(e: BlogError) -> Self {
        return ProgramError::Custom(e as u32);
    }
}