import {
  Keypair,
  Connection,
  PublicKey,
  LAMPORTS_PER_SOL,
  SystemProgram,
  TransactionInstruction,
  Transaction,
  sendAndConfirmTransaction,
  Account,
} from '@solana/web3.js';
import path from 'path';
import { serialize, deserialize, deserializeUnchecked } from "borsh";
import {getPayer, getRpcUrl, createKeypairFromFile} from './utils';
import * as borsh from "@project-serum/borsh";

// Define Account To Trade Against
const tradeAccount = new PublicKey("8VYN1kmMFm6PTwZtGUfx9P3kBdnHYpnjGFqtYXqr5v42"); // Enter Trade Account from 'npm run maker'

// Run Client
async function main() {

  // Connect
  const RPC_URL = "http://127.0.0.1:8899"
  const connection = new Connection(RPC_URL, "confirmed");

  // Wallet(s)
  const wallet = await getPayer();

  // Extract Program ID Address
  const PROGRAM_PATH = path.resolve(__dirname, '../../program/target/deploy/');
  const PROGRAM_KEYPAIR_PATH = path.join(PROGRAM_PATH, 'trade-keypair.json');
  const programKeypair = await createKeypairFromFile(PROGRAM_KEYPAIR_PATH);
  const PROGRAM_ID: PublicKey = programKeypair.publicKey;

  // Get Trade Claim Details
  let lamports = await getAccountDetails(connection, tradeAccount, "lamports");
  let orderStatus = await getAccountDetails(connection, tradeAccount, "orderStatus");
  const maker = await getAccountDetails(connection, tradeAccount, "maker");
  const taker = await getAccountDetails(connection, tradeAccount, "taker");
  console.log("Claimable SOL: ", lamports);
  console.log("Order Status: ", orderStatus);

  // Show User Account Info After Trade to Check Lamports
  const walletInfo = await connection.getAccountInfo(wallet.publicKey);
  if (walletInfo?.lamports) {
    console.log("Wallet SOL: ", walletInfo?.lamports / LAMPORTS_PER_SOL);
  }


  // Determine Instruction Accounts
  let ixAccounts = [
    {pubkey: wallet.publicKey, isSigner: true, isWritable: false},
    {pubkey: tradeAccount, isSigner: false, isWritable: true},
    {pubkey: maker, isSigner: false, isWritable: false}, // All Accounts being used have to be passed in
    {pubkey: taker, isSigner: false, isWritable: false}, // All Accounts being used have to be passed in
  ];

  // Call Transaction
  const ix = new TransactionInstruction({
    keys: ixAccounts,
    programId: PROGRAM_ID,
    data: Buffer.from(new Uint8Array([2, 0])), // 0 provided to supply 'rest' expectation after tag
  });

  // Send  Instruction
  await sendAndConfirmTransaction(
    connection,
    new Transaction().add(ix),
    [wallet],
  );

  // Get Trade Claim Details
  lamports = await getAccountDetails(connection, tradeAccount, "lamports");
  orderStatus = await getAccountDetails(connection, tradeAccount, "orderStatus");
  console.log("Claimable SOL after Claim: ",  lamports);
  console.log("Order Status after Claim: ",  orderStatus);

  // Show User Account Info After Trade to Check Lamports
  const walletInfo2 = await connection.getAccountInfo(wallet.publicKey);
  if (walletInfo2?.lamports) {
    console.log("Wallet SOL After Claim: ", walletInfo2?.lamports / LAMPORTS_PER_SOL);
  }
}

/**
  VIEW ACCOUNT DATA ////////////////////////////////////////////////////
 */
// View Trade Account
async function getAccountDetails(connection: Connection, account: PublicKey, retrieve: string) {

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
    borsh.u32("benchmark_price"),
    borsh.u32("closing_price"),
    borsh.u8("order_status"),
  ]);

  // Get Account Info
  const tradeAccountInfo = await connection.getAccountInfo(account);
  if (tradeAccountInfo) {
    // Decode Data
    const tradeAccountData = TRADE_ACCOUNT_DATA_LAYOUT.decode(tradeAccountInfo.data);

    // Return Lamports
    if (retrieve == "lamports") {
      return tradeAccountInfo.lamports / LAMPORTS_PER_SOL;
    }

    // Return Maker
    if (retrieve == "maker") {
      return tradeAccountData.maker;
    }

    // Return Taker
    if (retrieve == "taker") {
      return tradeAccountData.taker;
    }

    // Return Order Status
    if (retrieve == "orderStatus") {
      return tradeAccountData.order_status;
    }
  }
};

// Execute Program
main();
