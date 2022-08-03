use chainlink_solana as chainlink;

use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
  sysvar::{rent::Rent, Sysvar},
  borsh::try_from_slice_unchecked,
  account_info::{AccountInfo, next_account_info},
  entrypoint::ProgramResult, 
  pubkey::Pubkey,
  msg,
  program_error::ProgramError, system_instruction, program::{invoke_signed},
  clock::Clock,
};
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
        TradeInstruction::CreateUser => {
            msg!("Instruction: Creating User Program Derived Account");
            Self::create_user_pda(program_id, accounts)
        },
        TradeInstruction::WithdrawBalance => {
          msg!("Instruction: Withdrawing Balance");
          Self::withdraw_user_account_balance(program_id, accounts)
        },
        TradeInstruction::MakeTrade (trade) => {
            msg!("Instruction: Make Trade");
            Self::open_trade(program_id, accounts, trade)
        },
        TradeInstruction::TakeTrade (trade) => {
            msg!("Instruction: Take Trade");
            Self::take_trade(program_id, accounts, trade) // trade.direction (placeholder)
        },
        TradeInstruction::Claim => {
          msg!("Instruction: Claim Trade");
          Self::claim_trade(program_id, accounts)
      },
    }
  }


  // Create User
  fn create_user_pda(
    program_id: &Pubkey, 
    accounts: &[AccountInfo],
  ) -> ProgramResult {

    // Extract Accounts
    let account_info_iter = &mut accounts.iter();
    let authority_account = next_account_info(account_info_iter)?; // Holder = User
    let user_account = next_account_info(account_info_iter)?; // User Account Address (only Public Key Exists at this point)
    let system_program = next_account_info(account_info_iter)?; // Holder = Program (System Program)

    // Guard: Signer
    if !authority_account.is_signer {
      return Err(ProgramError::MissingRequiredSignature);
    }

    // Generate Program Derived Address (PDA)
    let (user_pda, user_bump) = Pubkey::find_program_address(
      &[b"user".as_ref(), authority_account.key.as_ref()],
      program_id 
    );

    // Guard: Ensure Account Key Received Matches PDA
    if user_pda != *user_account.key {
      return Err(TradeError::InvalidUserAccount.into())
    }

    // Calculate Lamports needed for PDA
    let rent = Rent::get()?;
    let rent_lamports = rent.minimum_balance(User::LEN);

    // Build Transaction for Trade PDA Account Creation
    let create_user_pda_ix = &system_instruction::create_account(
      authority_account.key,
      user_account.key,
      rent_lamports,
      User::LEN.try_into().unwrap(),
      program_id
    );

    // Create User Account (invoke signed as using PDA)
    msg!("Creating User account");
    invoke_signed(
      create_user_pda_ix,
      &[authority_account.clone(), user_account.clone(), system_program.clone()],
      &[&[b"user".as_ref(), authority_account.key.as_ref(), &[user_bump]]]
    )?;

    // Return Result
    msg!("User Account Created");
    Ok(())
  }

  
  // Withdraw Account Balance
  fn withdraw_user_account_balance(
    program_id: &Pubkey, 
    accounts: &[AccountInfo],
  ) -> ProgramResult {

    // Extract Accounts
    let account_info_iter = &mut accounts.iter();
    let authority_account = next_account_info(account_info_iter)?; // Holder = User (Users Main Wallet)
    let user_account = next_account_info(account_info_iter)?; // Holder = User (Users Trading Account)

    // Guard: Signer
    if !authority_account.is_signer {
      return Err(ProgramError::MissingRequiredSignature);
    }

    // Generate Program Derived Address (PDA)
    let (user_pda, _user_bump) = Pubkey::find_program_address(
      &[b"user".as_ref(), authority_account.key.as_ref()],
      program_id 
    );

    // Guard: Ensure Account Key Received Matches PDA
    if user_pda != *user_account.key {
      return Err(TradeError::InvalidUserAccount.into())
    }

     // Calculate Lamports needed for PDA ADD BACK IN PRODUCTION !!!!!!!!!!
     let rent = Rent::get()?;
     let rent_lamports = rent.minimum_balance(User::LEN);
    
    // Ensure enough to keep account open
    if **user_account.try_borrow_lamports()? < rent_lamports {
      return Err(TradeError::NotEnoughLamports.into());
    }

    // Specify withdrawal amount
    let tfer_amount = **user_account.lamports.borrow() - rent_lamports;

    // Debit from_account and credit to_account
    **user_account.try_borrow_mut_lamports()? -= tfer_amount;
    **authority_account.try_borrow_mut_lamports()? += tfer_amount;

    // Return Result
    Ok(())
  }


  // Claim Trade
  fn claim_trade(
    _program_id: &Pubkey,
    accounts: &[AccountInfo],
  ) -> ProgramResult {

    // Extract Accounts
    let account_info_iter = &mut accounts.iter();
    let claimer_authority_account = next_account_info(account_info_iter)?; // Holder = User
    let trade_account = next_account_info(account_info_iter)?; // Holder = Program (PDA) Created by Maker
    let maker_account = next_account_info(account_info_iter)?;
    let taker_account = next_account_info(account_info_iter)?;
    let _system_program = next_account_info(account_info_iter)?; // Not used but sent in as PDA Trade Account owner
    let chainlink_feed_account = next_account_info(account_info_iter)?; // Chainlink Price Feed Data Account
    let chainlink_program = next_account_info(account_info_iter)?; // Chainlink Program Account

    // Guard: Signer
    if !claimer_authority_account.is_signer {
      return Err(ProgramError::MissingRequiredSignature);
    }

    // Get Trade Account
    let mut trade_account_state = try_from_slice_unchecked::<Trade>(&trade_account.data.borrow())?;

    // Guard: Ensure Trade is in InPlay status (i.e. not claimed by started)
    if trade_account_state.order_status != 2 {
      msg!("Trade not in correct order status");
      return Err(TradeError::InvalidTradeForClaim.into())
    }

    // Guard: Ensure Trade Account Details Match Maker
    if trade_account_state.maker != *maker_account.key {
      msg!("Maker Details do Not Match");
      return Err(TradeError::InvalidTradeAccount.into())
    }

    // Guard: Ensure Trade Account Details Match Taker
    if trade_account_state.taker != *taker_account.key {
      msg!("Maker Details do Not Match");
      return Err(TradeError::InvalidTradeAccount.into())
    }

    // Get Clock
    let clock = Clock::get()?;
    let unix_current = clock.unix_timestamp as u32;
  
    // Guard: Time Check
    if unix_current < trade_account_state.unix_end {
      msg!("Too early to claim funds. Wait for contract to expire.");
      return Err(TradeError::InvalidTimeForClaim.into())
    }

    // Get Chainlink Price - // REMEMBER ONLY WORKS ON DEVNET
    let chainlink_round = chainlink::latest_round_data(chainlink_program.clone(), chainlink_feed_account.clone())?;
    let chainlink_asset_description = chainlink::description(chainlink_program.clone(), chainlink_feed_account.clone())?;
    let _chainlink_decimals = chainlink::decimals(chainlink_program.clone(), chainlink_feed_account.clone())?;
    let chainlink_price = chainlink_round.answer;

    // Guard: Ensure Asset Matches on Trade Account
    if chainlink_asset_description != trade_account_state.symbol {
      msg!("Chainlink Symbol Mismatch to Account Provided");
      return Err(TradeError::ChainlinkMismatch.into())
    }

    // Guard: Ensure Chainlink Price
    if chainlink_price == 0 {
      msg!("Chainlink Price Received as Zero. Chainlink Data Issue.");
      return Err(TradeError::ChainlinkDataIssue.into())
    }

    // Determine Winner
    let winner: String;
    let mut payee = maker_account;
    let unix_thresh = trade_account_state.unix_end + (1 * 24 * 60 * 60); // 24 hour allowance
    if (trade_account_state.direction == 0) & (chainlink_price > trade_account_state.benchmark_price) {
      winner = String::from("Maker");
    } else if (trade_account_state.direction == 1) & (chainlink_price < trade_account_state.benchmark_price) {
      winner = String::from("Maker");
    } else if (chainlink_price == trade_account_state.benchmark_price) || (unix_current >= unix_thresh) {
      winner = String::from("Draw");
    } else {
      winner = String::from("Taker");
      payee = taker_account;
    }

    // Update Trade Account
    trade_account_state.order_status = 3;
    trade_account_state.closing_price = chainlink_price;
    trade_account_state.serialize(&mut &mut trade_account.data.borrow_mut()[..])?;

    // Calculate Lamports needed for PDA
    let rent = Rent::get()?;
    let rent_lamports = rent.minimum_balance(Trade::LEN);

    // Pay Winner
    let tfer_amount = **trade_account.lamports.borrow() - rent_lamports;
    msg!("Lamports to Transfer: {:?}", tfer_amount);

    **trade_account.try_borrow_mut_lamports()? -= tfer_amount;
    if winner != "Draw" {
      **payee.try_borrow_mut_lamports()? += tfer_amount;
    } else {
      **maker_account.try_borrow_mut_lamports()? += tfer_amount / 2;
      **taker_account.try_borrow_mut_lamports()? += tfer_amount / 2;
    }

    // Return Result
    Ok(())
  }
  

  // Take Trade
  fn take_trade(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    _trade: Take,
  ) -> ProgramResult {

    // Extract Accounts
    let account_info_iter = &mut accounts.iter();
    let taker_authority_account = next_account_info(account_info_iter)?; // Holder = User
    let user_account = next_account_info(account_info_iter)?; // Holder = Program (PDA) 
    let trade_account = next_account_info(account_info_iter)?; // Holder = Program (PDA) Created by Maker
    let _system_program = next_account_info(account_info_iter)?; // Noy used but sent in as PDA Trade Account owner
    let chainlink_feed_account = next_account_info(account_info_iter)?; // Chainlink Price Feed Data Account
    let chainlink_program = next_account_info(account_info_iter)?; // Chainlink Program Account

    // Guard: Signer
    if !taker_authority_account.is_signer {
      return Err(ProgramError::MissingRequiredSignature);
    }

    // Get Clock
    let clock = Clock::get()?;
    let unix_start = clock.unix_timestamp as u32;
    let mut unix_end = unix_start;

    // Get Trade Account
    let mut trade_account_state = try_from_slice_unchecked::<Trade>(&trade_account.data.borrow())?;

    // Guard: Ensure A Match is Not Already Existing
    if trade_account_state.order_status != 1 {
      return Err(TradeError::AlreadyExistingTrade.into())
    }

    // Generate Program Derived Address (PDA) - To check user is payer
    let (user_pda, _user_bump) = Pubkey::find_program_address(
      &[b"user".as_ref(), taker_authority_account.key.as_ref()],
      program_id 
    );

    // Guard: Ensure Account Key Received Matches PDA - To check user is payer
    if user_pda != *user_account.key {
      return Err(TradeError::InvalidUserAccount.into())
    }

    // Calculate Trade Lamports for Trade Contracts
    let trade_lamports: u64;
    match trade_account_state.contract_size {
      0 => {trade_lamports = 1_000_000_00}, // 0.1 SOL
      1 => {trade_lamports = 1_000_000_000}, // 1 SOL
      5 => {trade_lamports = 5_000_000_000}, // 5 SOL
      _ => return Err(TradeError::InvalidContractSize.into())
    }

    // Get Chainlink Price -  // REMEMBER ONLY WORKS ON DEVNET
    let chainlink_round = chainlink::latest_round_data(chainlink_program.clone(), chainlink_feed_account.clone())?;
    let chainlink_asset_description = chainlink::description(chainlink_program.clone(), chainlink_feed_account.clone())?;
    let _chainlink_decimals = chainlink::decimals(chainlink_program.clone(), chainlink_feed_account.clone())?;
    let chainlink_price = chainlink_round.answer;

    // Guard: Ensure Asset Matches on Trade Account
    if chainlink_asset_description != trade_account_state.symbol {
      msg!("Chainlink Symbol Mismatch to Account Provided");
      return Err(TradeError::ChainlinkMismatch.into())
    }

    // Guard: Ensure Chainlink Price
    if chainlink_price == 0 {
      msg!("Chainlink Price Received as Zero. Chainlink Data Issue.");
      return Err(TradeError::ChainlinkDataIssue.into())
    }

    // Update Trade Account
    trade_account_state.taker = *user_account.key;
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
    let mut user_account_state = User::try_from_slice(&user_account.data.borrow())?;
    user_account_state.trades_placed += 1;
    user_account_state.serialize(&mut &mut user_account.data.borrow_mut()[..])?;

    // Guard: Transfer Lamports check
    msg!("Lamports being sent: {:?}", trade_lamports);
    if **user_account.try_borrow_lamports()? < trade_lamports {
      msg!("Not enough SOL (lamports)");
      return Err(TradeError::NotEnoughLamports.into());
    }

    // Transfer Lamports
    **user_account.try_borrow_mut_lamports()? -= trade_lamports; // Not owned by Program (thus Signed)
    **trade_account.try_borrow_mut_lamports()? += trade_lamports; // Owner by Program

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
    let user_account = next_account_info(account_info_iter)?; // Holder = Program (PDA)
    let trade_account = next_account_info(account_info_iter)?; // Holder = Program (PDA)
    let system_program = next_account_info(account_info_iter)?; // Holder = Program (System Program)
    let chainlink_feed_account = next_account_info(account_info_iter)?; // Chainlink Price Feed Data Account
    let chainlink_program = next_account_info(account_info_iter)?; // Chainlink Program Account

    // Guard: Signer
    if !authority_account.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    // Guard: User Account Owner
    if user_account.owner != program_id {
      msg!("Greeted account does not have the correct program id");
      return Err(ProgramError::IncorrectProgramId);
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

    // Get Chainlink Description - // REMEMBER ONLY WORKS ON DEVNET
    let chainlink_asset_description = chainlink::description(chainlink_program.clone(), chainlink_feed_account.clone())?;
    msg!("Chainlink Asset Description: {:?}", &chainlink_asset_description);
    msg!("Trade Symbol Description: {:?}", &trade.symbol);

    // Guard: Ensure Asset Matches on Trade Account
    if chainlink_asset_description != trade.symbol {
      msg!("Chainlink Symbol Mismatch to Account Provided");
      return Err(TradeError::ChainlinkMismatch.into())
    }

    // Calculate Lamports needed for PDA
    let rent = Rent::get()?;
    let rent_lamports = rent.minimum_balance(Trade::LEN);

    // Build Transaction for Trade PDA Account Creation
    let create_trade_pda_ix = &system_instruction::create_account(
      authority_account.key,
      trade_account.key,
      rent_lamports,
      Trade::LEN.try_into().unwrap(),
      program_id
    );

    // Create Trade Account (invoke signed as using PDA)
    msg!("Creating Trade account");
    invoke_signed(
      create_trade_pda_ix, 
      &[authority_account.clone(), trade_account.clone(), system_program.clone()],
      &[&[b"trade".as_ref(), trade.slug.as_ref(), authority_account.key.as_ref(), &[trade_bump]]]
    )?;

    // Get Clock
    let clock = Clock::get()?;
    let unix_created = clock.unix_timestamp as u32;

    // Get Current Trade Account State
    // Use Unchecked if working with Strings
    let mut trade_account_state = try_from_slice_unchecked::<Trade>(&trade_account.data.borrow())?;

    // Update Trade Account Information
    trade_account_state.maker = *user_account.key;
    trade_account_state.trade_account = *trade_account.key;
    trade_account_state.bump = trade_bump;
    trade_account_state.slug = trade.slug;
    trade_account_state.unix_created = unix_created;
    trade_account_state.symbol = trade.symbol;
    trade_account_state.contract_size = trade.contract_size;
    trade_account_state.direction = trade.direction;
    trade_account_state.duration = trade.duration;
    trade_account_state.order_status = 1; // see state

    // Update State with New Trade
    msg!("Serializing Trade data");
    trade_account_state.serialize(&mut &mut trade_account.data.borrow_mut()[..])?;

    // Update User Account State
    let mut user_account_state = User::try_from_slice(&user_account.data.borrow())?;
    user_account_state.trades_placed += 1;
    user_account_state.serialize(&mut &mut user_account.data.borrow_mut()[..])?;

    // Fund the Trade from User Account
    // Calculate Trade Lamports for Trade Contracts
    let trade_lamports: u64;
    match trade.contract_size {
      0 => {trade_lamports = 1_000_000_00}, // 0.1 SOL
      1 => {trade_lamports = 1_000_000_000}, // 1 SOL
      5 => {trade_lamports = 5_000_000_000}, // 5 SOL
      _ => return Err(TradeError::InvalidContractSize.into())
    }

    // Generate Program Derived Address (PDA)
    let (user_pda, _user_bump) = Pubkey::find_program_address(
      &[b"user".as_ref(), authority_account.key.as_ref()],
      program_id 
    );

    // Guard: Ensure Account Key Received Matches PDA
    if user_pda != *user_account.key {
      return Err(TradeError::InvalidUserAccount.into())
    }

    // Ensure enough to keep account open
    if **user_account.try_borrow_lamports()? < trade_lamports {
      return Err(TradeError::NotEnoughLamports.into());
    }

    // Debit from_account and credit to_account
    **user_account.try_borrow_mut_lamports()? -= trade_lamports;
    **trade_account.try_borrow_mut_lamports()? += trade_lamports;

    // Return
    msg!("Open Order Created");
    Ok(())
  }

} 
