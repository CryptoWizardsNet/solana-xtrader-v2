import {
  Connection,
  PublicKey,
  SystemProgram,
  TransactionInstruction,
  Transaction,
  sendAndConfirmTransaction,
} from '@solana/web3.js';
import {getKeypair, getProgramId, SELECTED_RPC_URL} from './utils';
import * as borsh from "@project-serum/borsh";


// Connect
const connection = new Connection(SELECTED_RPC_URL, "confirmed");

// Run Client
async function main() {

  // Get maker or take account
  const arg = process.argv.slice(2).toString();

  // Wallet(s)
  const wallet = await getKeypair(arg); 

  // Extract Program ID Address
  const PROGRAM_ID = await getProgramId();

  // Generate PDA - Trade
  const [userAccount] = await PublicKey.findProgramAddress(
    [Buffer.from("user"), wallet.publicKey.toBuffer()],
    PROGRAM_ID
  );

  // Get User Account Current Lamports
  const userAccountInfo = await connection.getAccountInfo(userAccount);
  console.log("User Account Lamports: ", userAccountInfo?.lamports);

  // Confirm Accounts
  console.log('Authority Account: ', wallet.publicKey.toBase58());
  console.log('User Account: ', userAccount.toBase58());

  // Determine Instruction Accounts
  let ixAccounts = [
    {pubkey: wallet.publicKey, isSigner: true, isWritable: false},
    {pubkey: userAccount, isSigner: false, isWritable: true},
  ];

  // Call Transaction
  const ix = new TransactionInstruction({
    keys: ixAccounts,
    programId: PROGRAM_ID,
    data: Buffer.from(new Uint8Array([1, 0])), // 1 = Withdraw All Except Rent, Other 0 is due to 'rest' formula used in Program
  });

  // Send  Instruction
  await sendAndConfirmTransaction(
    connection,
    new Transaction().add(ix),
    [wallet],
  );

  // View Results
  await viewUserAccount(userAccount);
};

/**
  VIEW ACCOUNT DATA ////////////////////////////////////////////////////
 */
// View Trade Account
async function viewUserAccount(account: PublicKey) {

  // Define Post Account Structure
  const TRADE_ACCOUNT_DATA_LAYOUT = borsh.struct([
    borsh.u32("trades_placed"),
  ]);

  // Get User Account Current Info
  const userAccountInfo = await connection.getAccountInfo(account);
  console.log("User Account Lamports: ", userAccountInfo?.lamports);

  // Decode and Show Post Account
  if (userAccountInfo) { 
    const userAccountData = TRADE_ACCOUNT_DATA_LAYOUT.decode(userAccountInfo.data);
    console.log("User Account Info: \n", userAccountData);
  };
};

// Run Main
main();
