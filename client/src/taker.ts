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

// Structure for Blog Instruction
class TakeIx {
  tag: number; direction: number;
  constructor(tag: number, direction: number) {
    this.tag = tag;
    this.direction = direction;
  }
}

// Define Account To Trade Against
const tradeAccount = new PublicKey("J932LWXxgYyranC6YQbbb7DLNjnxnyQ5Af8aoCZaSmP1"); // Enter Trade Account from 'npm run maker'

// Connect
const connection = new Connection(SELECTED_RPC_URL, "confirmed");

// Run Client
async function main() {

  // Wallet(s)
  const wallet = await getKeypair("taker");
  console.log(wallet.publicKey.toBase58());

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
  const tradeIx = new TakeIx(3, 0); // Take
  const schema = new Map([[TakeIx, { kind: 'struct', fields: [['tag', 'u8'], ['direction', 'u8']]}]]);
  const instruction_data = serialize(schema, tradeIx);
  console.log("Instruction Data: ", instruction_data.length);

  // System Program (Needed for PDA Creation in Program)
  const systemProgramId = SystemProgram.programId;

  // Determine Instruction Accounts
  let ixAccounts = [
    {pubkey: wallet.publicKey, isSigner: true, isWritable: true},
    {pubkey: userAccount, isSigner: false, isWritable: true},
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
    borsh.publicKey("trade_account"),
    borsh.u8("bump"),
    borsh.str("slug"),
    borsh.str("symbol"),
    borsh.u8("content"),
    borsh.u8("direction"),
    borsh.u8("duration"),
    borsh.u32("unix_created"),
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

// Run Main
main();
