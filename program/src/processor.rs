use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
  sysvar::{rent::Rent, Sysvar},
  borsh::try_from_slice_unchecked,
  account_info::{AccountInfo, next_account_info},
  entrypoint,
  entrypoint::ProgramResult, 
  pubkey::Pubkey,
  msg,
  program_error::ProgramError, system_instruction, program::invoke_signed,
};
use std::mem;
use crate::instruction::{BlogInstruction};
use crate::state::{Blog, Post};
use crate::error::BlogError;


// PROCESSOR
pub struct Processor;
impl Processor {

  // Route Incoming Request
  pub fn process(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
  ) -> ProgramResult {

    // Confirm initialization
    msg!("Process starting...");

    // Unpack Instruction
    let instruction = BlogInstruction::unpack(instruction_data)?;
    msg!("Instruction Received: {:?}", &instruction);

    // Route Instruction
    match instruction {
        BlogInstruction::InitBlog {} => {
            msg!("Instruction: InitBlog");
            Self::process_init_blog(accounts, program_id)
        },
        BlogInstruction::CreatePost { slug, title, content} => {
            msg!("Instruction: CreatePost");
            Self::process_create_post(accounts, slug, title, content, program_id)
        }
    }
  }

  // Create POST
  fn process_create_post(
    accounts: &[AccountInfo],
    slug: String,
    title: String,
    content: String,
    program_id: &Pubkey
  ) -> ProgramResult {

    // Guard: Ensure character count not exceeded
    if slug.len() > 10 || content.len() > 20 || title.len() > 50 {
        return Err(BlogError::InvalidPostData.into())
    }

    // Get Accounts
    let account_info_iter = &mut accounts.iter();
    let authority_account = next_account_info(account_info_iter)?;
    let blog_account = next_account_info(account_info_iter)?;
    let post_account = next_account_info(account_info_iter)?;
    let system_program = next_account_info(account_info_iter)?;

    // Guard: Ensure Signer
    if !authority_account.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    // Generate Program Derived Address (PDA) for Blog
    let (blog_pda, _blog_bump) = Pubkey::find_program_address(
        &[b"blog".as_ref(), authority_account.key.as_ref()],
        program_id
    );

    // Guard: Ensure PDA Matches account and Blog Account is Writable and has been initialized
    if blog_pda != *blog_account.key || !blog_account.is_writable || blog_account.data_is_empty() {
        return Err(BlogError::InvalidBlogAccount.into())
    }

    // Generate Program Derived Address (PDA) for Post
    let (post_pda, post_bump) = Pubkey::find_program_address(
        &[b"post".as_ref(), slug.as_ref(), authority_account.key.as_ref()],
        program_id
    );

    // Guard: Ensure Post Account matches account key
    if post_pda != *post_account.key {
        return Err(BlogError::InvalidPostAccount.into())
    }

    // Calculate Rent Exempt
    let post_len = 32 + 32 + 1 + (4 + slug.len()) + (4 + title.len()) + (4 + content.len());
    let rent = Rent::get()?;
    let rent_lamports = rent.minimum_balance(post_len);

    // Build Transaction
    let create_post_pda_ix = &system_instruction::create_account(
        authority_account.key,
        post_account.key,
        rent_lamports,
        post_len.try_into().unwrap(),
        program_id
    );

    // Create Post Account (invoke signed as using PDA)
    msg!("Creating post account!");
    invoke_signed(
        create_post_pda_ix, 
        &[authority_account.clone(), post_account.clone(), system_program.clone()],
        &[&[b"post".as_ref(), slug.as_ref(), authority_account.key.as_ref(), &[post_bump]]]
    )?;

    // Get Current Post Account State
    let mut post_account_state = try_from_slice_unchecked::<Post>(&post_account.data.borrow()).unwrap();

    // Make State Adjustments
    post_account_state.author = *authority_account.key;
    post_account_state.blog = *blog_account.key;
    post_account_state.bump = post_bump;
    post_account_state.slug = slug;
    post_account_state.title = title;
    post_account_state.content = content;

    // Update State with New Post
    msg!("Serializing Post data");
    post_account_state.serialize(&mut &mut post_account.data.borrow_mut()[..])?;

    // Get Blog Account State
    let mut blog_account_state = Blog::try_from_slice(&blog_account.data.borrow())?;
    blog_account_state.post_count += 1;

    // Update Blog State with latest adjustment
    msg!("Serializing Blog data");
    blog_account_state.serialize(&mut &mut blog_account.data.borrow_mut()[..])?;

    // Return Result
    Ok(())
  }

  // CREATE BLOG
  fn process_init_blog(
    accounts: &[AccountInfo],
    program_id: &Pubkey
  ) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();
    let authority_account = next_account_info(account_info_iter)?;
    let blog_account = next_account_info(account_info_iter)?;
    let system_program = next_account_info(account_info_iter)?;

    // Guard: Signer
    if !authority_account.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    // Generate Program Derived Address (PDA)
    let (blog_pda, blog_bump) = Pubkey::find_program_address(
        &[b"blog".as_ref(), authority_account.key.as_ref()],
        program_id 
    );

    // Guard: Ensure Account Key Received Matches PDA
    if blog_pda != *blog_account.key {
        return Err(BlogError::InvalidBlogAccount.into())
    }

    // Calculate Lamports needed for PDA
    let rent = Rent::get()?;
    let rent_lamports = rent.minimum_balance(Blog::LEN);
    
    // Build Transaction for Account Creation
    let create_blog_pda_ix = &system_instruction::create_account(
        authority_account.key,
        blog_account.key,
        rent_lamports,
        Blog::LEN.try_into().unwrap(),
        program_id
    );

    // Create Blog Account (invoke signed as using PDA)
    msg!("Creating blog account!");
    invoke_signed(
        create_blog_pda_ix, 
        &[authority_account.clone(), blog_account.clone(), system_program.clone()],
        &[&[b"blog".as_ref(), authority_account.key.as_ref(), &[blog_bump]]]
    )?;

    // Get Current Blog Account State
    let mut blog_account_state = Blog::try_from_slice(&blog_account.data.borrow())?;

    // Make Blog Account State Adjustments
    blog_account_state.authority = *authority_account.key;
    blog_account_state.bump = blog_bump;
    blog_account_state.post_count = 0;

    // Update To New State
    blog_account_state.serialize(&mut &mut blog_account.data.borrow_mut()[..])?;

    // Return Result
    Ok(())
  }
} 
