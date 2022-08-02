import {
  Connection,
  PublicKey,
  SystemProgram,
  LAMPORTS_PER_SOL,
  Transaction,
  sendAndConfirmTransaction,
} from '@solana/web3.js';
import {getKeypair, getProgramId, SELECTED_RPC_URL} from './utils';


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

  // Confirm Accounts
  console.log('Tfer From Authority Account: ', wallet.publicKey.toBase58());
  console.log('Tfer To User Account: ', userAccount.toBase58());

  const transaction = new Transaction().add(
    SystemProgram.transfer({
        fromPubkey: wallet.publicKey,
        toPubkey: userAccount,
        lamports: LAMPORTS_PER_SOL * 1, // Send x1 SOL
    })
  );

  // Sign transaction, broadcast, and confirm
  const signature = await sendAndConfirmTransaction(
    connection,
    transaction,
    [wallet]
  );

  // Get Post Account Current Info
  const userAccountInfo = await connection.getAccountInfo(userAccount);
  console.log("Account Lamports: ", userAccountInfo?.lamports);
};


// Run Main
main();
