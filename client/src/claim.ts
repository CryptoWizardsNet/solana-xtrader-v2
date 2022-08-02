import {
  Connection,
  PublicKey,
  LAMPORTS_PER_SOL,
  SystemProgram,
  TransactionInstruction,
  Transaction,
  sendAndConfirmTransaction,
} from '@solana/web3.js';
import {getKeypair, getProgramId, accountChainlinkPriceFeed, accountChainlinkProgramOwner, SELECTED_RPC_URL} from './utils';
import * as borsh from "@project-serum/borsh";

// Define Account To Trade Against
const tradeAccount = new PublicKey("J932LWXxgYyranC6YQbbb7DLNjnxnyQ5Af8aoCZaSmP1"); // Enter Trade Account from 'npm run maker'

// Connect
const connection = new Connection(SELECTED_RPC_URL, "confirmed");

// Run Client
async function main() {

  // Wallet(s)
  const wallet = await getKeypair("maker"); 

  // Extract Program ID Address
  const PROGRAM_ID = await getProgramId();

  // Generate PDA - Trade
  const [userAccount] = await PublicKey.findProgramAddress(
    [Buffer.from("user"), wallet.publicKey.toBuffer()],
    PROGRAM_ID
  );

  // Get Trade Claim Details
  let lamports = await getAccountDetails(connection, tradeAccount, "lamports");
  let orderStatus = await getAccountDetails(connection, tradeAccount, "orderStatus");
  const maker = await getAccountDetails(connection, tradeAccount, "maker");
  const taker = await getAccountDetails(connection, tradeAccount, "taker");
  console.log("Claimable SOL: ", lamports);
  console.log("Order Status: ", orderStatus);

  // Show User Account Info After Trade to Check Lamports
  const walletInfo = await connection.getAccountInfo(userAccount);
  if (walletInfo?.lamports) {
    console.log("User Account SOL: ", walletInfo?.lamports / LAMPORTS_PER_SOL);
  }

  // System Program (Needed for PDA Creation in Program)
  const systemProgramId = SystemProgram.programId;

  // Determine Instruction Accounts
  let ixAccounts = [
    {pubkey: wallet.publicKey, isSigner: true, isWritable: false},
    {pubkey: tradeAccount, isSigner: false, isWritable: true},
    {pubkey: maker, isSigner: false, isWritable: true}, // All Accounts being used have to be passed in
    {pubkey: taker, isSigner: false, isWritable: true}, // All Accounts being used have to be passed in
    {pubkey: systemProgramId, isSigner: false, isWritable: false}, // Needed as PDA balance will change
    {pubkey: accountChainlinkPriceFeed, isSigner: false, isWritable: false},
    {pubkey: accountChainlinkProgramOwner, isSigner: false, isWritable: false},
  ];

  console.log("tradeAccount ", tradeAccount.toBase58());
  console.log("userAccount ", userAccount.toBase58());
  console.log("maker ", maker.toBase58());
  console.log("taker ", taker.toBase58());
  console.log("systemProgramId ", systemProgramId.toBase58());
  console.log("accountChainlinkPriceFeed ", accountChainlinkPriceFeed.toBase58());
  console.log("accountChainlinkProgramOwner ", accountChainlinkProgramOwner.toBase58());

  // Call Transaction
  const ix = new TransactionInstruction({
    keys: ixAccounts,
    programId: PROGRAM_ID,
    data: Buffer.from(new Uint8Array([4, 0])), // 4 = Claim, 0 provided to supply 'rest' expectation after tag
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
  const walletInfo2 = await connection.getAccountInfo(userAccount);
  if (walletInfo2?.lamports) {
    console.log("User Account SOL After Claim: ", walletInfo2?.lamports / LAMPORTS_PER_SOL);
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
    borsh.i128("benchmark_price"),
    borsh.i128("closing_price"),
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
