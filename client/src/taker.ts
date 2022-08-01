import {
  Keypair,
  Connection,
  PublicKey,
  SystemProgram,
  TransactionInstruction,
  Transaction,
  sendAndConfirmTransaction,
} from '@solana/web3.js';
import path from 'path';
import { serialize } from "borsh";
import {getKeypair, createKeypairFromFile, accountChainlinkPriceFeed, accountChainlinkProgramOwner, SELECTED_RPC_URL} from './utils';
import * as borsh from "@project-serum/borsh";

// Structure for Blog Instruction
class TakeIx {
  tag: number; direction: number;
  constructor(tag: number, direction: number) {
    this.tag = tag;
    this.direction = direction;
  }
}

// Define Account To Trade Against
const tradeAccount = new PublicKey("GJ2Q1RtYF75MtkvP4vnmCAxS36UGQ8n3easBpDZpNEDc"); // Enter Trade Account from 'npm run maker'

// Connect
const connection = new Connection(SELECTED_RPC_URL, "confirmed");

// Run Client
async function main() {

  // Wallet(s)
  const wallet = await getKeypair("taker");
  console.log(wallet.publicKey.toBase58());

  // Extract Program ID Address
  const PROGRAM_PATH = path.resolve(__dirname, '../../program/target/deploy/');
  const PROGRAM_KEYPAIR_PATH = path.join(PROGRAM_PATH, 'trade-keypair.json');
  const programKeypair = await createKeypairFromFile(PROGRAM_KEYPAIR_PATH);
  const PROGRAM_ID: PublicKey = programKeypair.publicKey;

  // Structure User Account Tx
  const userSpace = 4; // Bytes
  const userAccount = Keypair.generate();
  const rentExemptionAmount = await connection.getMinimumBalanceForRentExemption(userSpace);
  const createUserAccountParams = {
    fromPubkey: wallet.publicKey,
    newAccountPubkey: userAccount.publicKey,
    lamports: rentExemptionAmount + 1100000000, // + 2 SOL Funding Account
    space: userSpace,
    programId: PROGRAM_ID,
  };

  // Create User Account
  const createAccountTransaction = new Transaction().add(SystemProgram.createAccount(createUserAccountParams));
  await sendAndConfirmTransaction(connection, createAccountTransaction, [wallet, userAccount,]);

  // Get User Account Trades Count (for slug)
  const userAccountInfo = await connection.getAccountInfo(userAccount.publicKey);
  const userTradesCount = userAccountInfo?.data.readUInt8();
  console.log("User Account Info Trades Count: ", userTradesCount);

  // Build Instruction for Blog with Post]
  const tradeIx = new TakeIx(1, 0); // Take, Go Long
  const schema = new Map([[TakeIx, { kind: 'struct', fields: [['tag', 'u8'], ['direction', 'u8']]}]]);
  const instruction_data = serialize(schema, tradeIx);
  console.log("Instruction Data: ", instruction_data.length);

  // System Program (Needed for PDA Creation in Program)
  const systemProgramId = SystemProgram.programId;

  // Determine Instruction Accounts
  let ixAccounts = [
    {pubkey: wallet.publicKey, isSigner: true, isWritable: true},
    {pubkey: userAccount.publicKey, isSigner: false, isWritable: true},
    {pubkey: tradeAccount, isSigner: false, isWritable: true},
    {pubkey: systemProgramId, isSigner: false, isWritable: false}, // Needed as PDA balance will change
    {pubkey: accountChainlinkPriceFeed, isSigner: false, isWritable: false},
    {pubkey: accountChainlinkProgramOwner, isSigner: false, isWritable: false},
  ];

  // Call Transaction
  const ix = new TransactionInstruction({
    keys: ixAccounts,
    programId: PROGRAM_ID,
    data: Buffer.from(instruction_data),
  });

  // Send  Instruction
  await sendAndConfirmTransaction(
    connection,
    new Transaction().add(ix),
    [wallet],
  );

  // View Results
  await viewTradeAccount(connection, tradeAccount);
  // await viewAllAccounts(connection, PROGRAM_ID);
};


/**
  VIEW ACCOUNT DATA ////////////////////////////////////////////////////
 */
// View Trade Account
async function viewTradeAccount(connection: Connection, account: PublicKey) {

  // Define Post Account Structure
  const TRADE_ACCOUNT_DATA_LAYOUT = borsh.struct([
    borsh.publicKey("maker"),
    borsh.publicKey("taker"),
    borsh.u8("bump"),
    borsh.str("slug"),
    borsh.str("symbol"),
    borsh.u8("content"),
    borsh.u8("direction"),
    borsh.u8("duration"),
    borsh.u32("unix_start"),
    borsh.u32("unix_end"),
    borsh.i128("benchmark_price"),
    borsh.i128("closing_price"),
    borsh.u8("order_status"),
  ]);

  // Get Post Account Current Info
  const tradeAccountInfo = await connection.getAccountInfo(account);

  // Decode and Show Post Account
  if (tradeAccountInfo) { 
    const postAccountData = TRADE_ACCOUNT_DATA_LAYOUT.decode(tradeAccountInfo.data);
    console.log("Trade Account Info: \n", postAccountData);
    console.log(postAccountData.taker.toBase58());
  };
};

// View all Program Owned Accounts
async function viewAllAccounts(connection: Connection, programId: PublicKey) {
  // "processed" | "confirmed" | "finalized" | "recent" | "single" | "singleGossip" | "root" | "max" | << Optional
  const accounts = await connection.getProgramAccounts(programId, "confirmed");
  console.log("Program Accounts: ", accounts);
}

// Run Main
main();
