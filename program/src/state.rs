use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{pubkey::Pubkey};

// Account Blog
#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
pub struct Blog {
    pub authority: Pubkey,
    pub bump: u8,
    pub post_count: u8 // 10 posts max
}

// Get LEN of Blog
impl Blog {
  pub const LEN: usize = 32 + 1 + 1;
}

// Account Post
#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
pub struct Post {
    pub author: Pubkey,
    pub blog: Pubkey,
    pub bump: u8,
    pub slug: String, // 10 chars max
    pub title: String, // 20 chars max
    pub content: String, // 50 chars max
}
