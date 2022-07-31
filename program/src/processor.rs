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
  clock::Clock,
};
use std::mem;
use crate::instruction::{TradeInstruction, Make, Take};
use crate::state::{User, Trade};
use crate::error::TradeError;


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
    let instruction = TradeInstruction::unpack(instruction_data)?;
    msg!("Instruction Received: {:?}", &instruction);

    // Route Instruction
    match instruction {
        TradeInstruction::MakeTrade (trade) => {
            msg!("Instruction: Make Trade");
            Self::open_trade(program_id, accounts, trade)
        },
        TradeInstruction::TakeTrade (trade) => {
            msg!("Instruction: Take Trade");
            Self::take_trade(program_id, accounts, trade)
        },
    }
  }

  
  // Take Trade
  fn take_trade(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    trade: Take,
  ) -> ProgramResult {

    // Extract Accounts
    let account_info_iter = &mut accounts.iter();
    let taker_authority_account = next_account_info(account_info_iter)?; // Holder = User
    let trade_account = next_account_info(account_info_iter)?; // Holder = Program (PDA) Created by Maker

    // Guard: Signer
    if !taker_authority_account.is_signer {
      return Err(ProgramError::MissingRequiredSignature);
    }

    // GET CHAINLINK PRICE HERE !!!!!!!!!
    let chainlink_price = 47;

    // // Get Clock !!! CHANGE FOR PRODUCTION !!!!!!!!!
    // let clock = Clock::get()?;
    let unix_start = 1659167706; /// !!! REMOVE IN PRODUCTION
    let mut unix_end = unix_start;

    // Get Trade Account
    let mut trade_account_state = try_from_slice_unchecked::<Trade>(&trade_account.data.borrow())?;
    msg!("Lamports Required: {:?}", trade_account_state);

    // ADD IN PRODUCTION !!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!
    // // Guard: Ensure A Match is Not Already Existing
    // if trade_account_state.order_status != 1 {
    //   return Err(TradeError::AlreadyExistingTrade.into())
    // }

    // Calculate Trade Lamports for Trade Contracts
    let trade_lamports: u64;
    match trade_account_state.contract_size {
      0 => {trade_lamports = 1_000_000_00}, // 0.1 SOL
      1 => {trade_lamports = 1_000_000_000}, // 1 SOL
      5 => {trade_lamports = 5_000_000_000}, // 5 SOL
      _ => return Err(TradeError::InvalidContractSize.into())
    }

    // Guard: Transfer Lamports check
    if **taker_authority_account.try_borrow_lamports()? < trade_lamports {
      return Err(TradeError::NotEnoughLamports.into());
    }

    // Transfer Lamports
    **taker_authority_account.try_borrow_mut_lamports()? -= trade_lamports;
    **trade_account.try_borrow_mut_lamports()? += trade_lamports;

    // Update Trade Account
    trade_account_state.taker = *taker_authority_account.key;
    trade_account_state.unix_start = unix_start;
    trade_account_state.benchmark_price = chainlink_price;
    trade_account_state.order_status = 2;
    trade_account_state.serialize(&mut &mut trade_account.data.borrow_mut()[..])?;
    match trade_account_state.duration {
      0 => {unix_end += 5 * 60},
      1 => {unix_end += 60 * 60},
      2 => {unix_end += 24 * 60 * 60},
      _ => {return Err(TradeError::InvalidDurationCalculation.into())}
    }
    trade_account_state.unix_end += unix_end;
    trade_account_state.serialize(&mut &mut trade_account.data.borrow_mut()[..])?;

    // Update User Account State
    let mut user_account_state = User::try_from_slice(&taker_authority_account.data.borrow())?;
    user_account_state.trades_placed += 1;
    user_account_state.serialize(&mut &mut taker_authority_account.data.borrow_mut()[..])?;

    // Return
    Ok(())
  }

  // Open Trade
  fn open_trade(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    trade: Make,
  ) -> ProgramResult {

    // extract Accounts
    let account_info_iter = &mut accounts.iter();
    let authority_account = next_account_info(account_info_iter)?; // Holder = User
    let trade_account = next_account_info(account_info_iter)?; // Holder = Program (PDA)
    let system_program = next_account_info(account_info_iter)?; // Holder = Program (System Program)
    
    // Guard: Signer
    if !authority_account.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    // Generate Program Derived Address (PDA)
    let (trade_pda, trade_bump) = Pubkey::find_program_address(
        &[b"trade".as_ref(), trade.slug.as_ref(), authority_account.key.as_ref()],
        program_id 
    );

    // Guard: Ensure Account Key Received Matches PDA
    if trade_pda != *trade_account.key {
      return Err(TradeError::InvalidTradeAccount.into())
    }

    // CHANGE ON DEPLOYMENT !!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!
    // // Calculate Lamports needed for PDA
    // let rent = Rent::get()?;
    // let rent_lamports = rent.minimum_balance(Trade::LEN);
    let mut rent_lamports: u64 = 100_000; // DELETE ON DEPLOYMENT !!!!!!!!!!!!!!!!!!!!!!!!!!!

    // Calculate Trade Lamports for Trade Contracts
    let trade_lamports: u64;
    match trade.contract_size {
      0 => {trade_lamports = 1_000_000_00}, // 0.1 SOL
      1 => {trade_lamports = 1_000_000_000}, // 1 SOL
      5 => {trade_lamports = 5_000_000_000}, // 5 SOL
      _ => return Err(TradeError::InvalidContractSize.into())
    }

    // Add Trade Lamports to Rent Lamports
    rent_lamports += trade_lamports;

    // Build Transaction for Trade PDA Account Creation
    let create_trade_pda_ix = &system_instruction::create_account(
      authority_account.key,
      trade_account.key,
      rent_lamports,
      Trade::LEN.try_into().unwrap(),
      program_id
    );

    // Create Trade Account (invoke signed as using PDA)
    msg!("Creating Trade account!");
    invoke_signed(
      create_trade_pda_ix, 
        &[authority_account.clone(), trade_account.clone(), system_program.clone()],
        &[&[b"trade".as_ref(), trade.slug.as_ref(), authority_account.key.as_ref(), &[trade_bump]]]
    )?;

    // Get Current Trade Account State
    // Use Unchecked if working with Strings
    let mut trade_account_state = try_from_slice_unchecked::<Trade>(&trade_account.data.borrow())?;
    msg!("trade_account_state: {:?}", &trade_account_state);

    // Update Trade Account Information
    trade_account_state.maker = *authority_account.key;
    trade_account_state.bump = trade_bump;
    trade_account_state.symbol = trade.symbol;
    trade_account_state.contract_size = trade.contract_size;
    trade_account_state.direction = trade.direction;
    trade_account_state.duration = trade.duration;
    trade_account_state.benchmark_price = trade.benchmark_price;
    trade_account_state.order_status = 1; // see state

    // Update State with New Trade
    msg!("Serializing Trade data");
    trade_account_state.serialize(&mut &mut trade_account.data.borrow_mut()[..])?;

    // Update User Account State
    let mut user_account_state = User::try_from_slice(&authority_account.data.borrow())?;
    user_account_state.trades_placed += 1;
    user_account_state.serialize(&mut &mut authority_account.data.borrow_mut()[..])?;

    // Return
    msg!("Open Order Created");
    Ok(())
  }

} 
