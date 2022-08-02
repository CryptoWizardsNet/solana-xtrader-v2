import {
  Connection,
  PublicKey,
  SystemProgram,
  TransactionInstruction,
  Transaction,
  sendAndConfirmTransaction,
} from '@solana/web3.js';
import { serialize } from "borsh";
import {getKeypair, getProgramId, accountChainlinkPriceFeed, accountChainlinkProgramOwner, SELECTED_RPC_URL} from './utils';
import * as borsh from "@project-serum/borsh";

// DevNet Program Id: XX84Z5ozTcRcmvS72kw9PHmqqBNrYDHX9q21iu5jcSc

// Structure for Blog Instruction
class MakeIx {
  tag: number; symbol: string; slug: string; contract_size: number; direction: number; duration: number;
  constructor(tag: number, symbol: string, slug: string, contract_size: number, direction: number, duration: number) {
    this.tag = tag;
    this.symbol = symbol;
    this.slug = slug;
    this.contract_size = contract_size;
    this.direction = direction;
    this.duration = duration;
  }
}

// Connect
const connection = new Connection(SELECTED_RPC_URL, "confirmed");

// Run Client
async function main() {

  // Wallet(s)
  const wallet = await getKeypair("maker"); 
  console.log("Wallet Account: ", wallet.publicKey.toBase58());

  // Extract Program ID Address
  const PROGRAM_ID = await getProgramId();

  // Generate PDA - Trade
  const [userAccount] = await PublicKey.findProgramAddress(
    [Buffer.from("user"), wallet.publicKey.toBuffer()],
    PROGRAM_ID
  );

  // Get User Account Trades Count (for slug)
  const userAccountInfo = await connection.getAccountInfo(userAccount);
  const userTradesCount = userAccountInfo?.data.readUInt8();
  console.log("User Account: ", userAccount.toBase58());
  console.log("User Account Info Trades Count: ", userTradesCount);

  // Build Instruction for Blog with Post]
  const SLUGN = 7; // Increment each time you run for Post slug (as slug is used in PDA)
  const tradeIx = new MakeIx(2, "SOL / USD", 'trade' + SLUGN, 1, 1, 0); // Small Trade 0, Medium Trade 1, Large 2
  const schema = new Map([[MakeIx, { kind: 'struct', fields: [['tag', 'u8'], ['symbol', 'string'], ['slug', 'string'], ['contract_size', 'u8'], ['direction', 'u8'], ['duration', 'u8']]}]]);
  const instruction_data = serialize(schema, tradeIx);
  console.log("Instruction Data: ", instruction_data.length);

  // Generate PDA - Trade
  const [tradeAccount] = await PublicKey.findProgramAddress(
    [Buffer.from("trade"), Buffer.from(tradeIx.slug), wallet.publicKey.toBuffer()],
    PROGRAM_ID
  );

  // System Program (Needed for PDA Creation in Program)
  const systemProgramId = SystemProgram.programId;

  // Confirm Accounts
  console.log('Authority Account: ', wallet.publicKey.toBase58());
  console.log('Trade Account: ', tradeAccount.toBase58());
  console.log('System Program Account: ', systemProgramId.toBase58());

  // Determine Instruction Accounts
  let ixAccounts = [
    {pubkey: wallet.publicKey, isSigner: true, isWritable: false},
    {pubkey: userAccount, isSigner: false, isWritable: true},
    {pubkey: tradeAccount, isSigner: false, isWritable: true},
    {pubkey: systemProgramId, isSigner: false, isWritable: false},
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
  await viewTradeAccount(tradeAccount);
  // await viewAllAccounts(connection, PROGRAM_ID);
};

/**
  VIEW ACCOUNT DATA ////////////////////////////////////////////////////
 */
// View Trade Account
async function viewTradeAccount(account: PublicKey) {

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
