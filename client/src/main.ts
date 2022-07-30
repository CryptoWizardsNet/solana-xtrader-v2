import {
  Keypair,
  Connection,
  PublicKey,
  LAMPORTS_PER_SOL,
  SystemProgram,
  TransactionInstruction,
  Transaction,
  sendAndConfirmTransaction,
} from '@solana/web3.js';
import path from 'path';
import { serialize, deserialize, deserializeUnchecked } from "borsh";
import {getPayer, getRpcUrl, createKeypairFromFile} from './utils';
import * as borsh from "@project-serum/borsh";

// Structure for Blog Instruction
class BlogIx {
  i: number; s: string; y: string; z: string;
  constructor(i: number, s: string, y: string, z: string) {
      this.i = i; // Instruction
      this.s = s; // Slug
      this.y = y; // Title
      this.z = z; // Content
  }
}

// Run Client
async function main() {

  // Connect
  const RPC_URL = "http://127.0.0.1:8899"
  const connection = new Connection(RPC_URL, "confirmed");

  // Wallet(s)
  const wallet = await getPayer();

  // Extract Program ID Address
  const PROGRAM_PATH = path.resolve(__dirname, '../../program/target/deploy/');
  const PROGRAM_KEYPAIR_PATH = path.join(PROGRAM_PATH, 'blog-keypair.json');
  const programKeypair = await createKeypairFromFile(PROGRAM_KEYPAIR_PATH);
  const PROGRAM_ID: PublicKey = programKeypair.publicKey;

  // Build Instruction for Blog with Post]
  const POSTN = 1; // Increment each time you run for Post slug (as slug is used in PDA)
  const blogIx = new BlogIx(0, 'slug' + POSTN, 'My Title' + POSTN, 'My Content' + POSTN); // 0 for Blog Init, 1 for Create Post
  const schema = new Map([[BlogIx, { kind: 'struct', fields: [['i', 'u8'], ['s', 'string'], ['y', 'string'], ['z', 'string']] }]]);
  const instruction_data = serialize(schema, blogIx);
  console.log("Instruction Data: ", instruction_data);

  // Authority Account
  const authorityAccount = wallet.publicKey;

  // Generate PDA - Blog
  const [blogAccount] = await PublicKey.findProgramAddress(
    [Buffer.from("blog"), wallet.publicKey.toBuffer()],
    PROGRAM_ID
  );

  // Generate PDA - Post
  const [postAccount] = await PublicKey.findProgramAddress(
    [Buffer.from("post"), Buffer.from(blogIx.s), wallet.publicKey.toBuffer()],
    PROGRAM_ID
  );

  // System Program
  const systemProgramId = SystemProgram.programId;

  // Confirm Accounts
  console.log('Authority Account: ', authorityAccount.toBase58());
  console.log('Blog Account: ', blogAccount.toBase58());
  console.log('Post Account: ', postAccount.toBase58());
  console.log('System Program Account: ', systemProgramId.toBase58());

  // Determine Instruction Accounts
  // Only including Post Account if doing a transaction to Post
  let ixAccounts = [{pubkey: authorityAccount, isSigner: true, isWritable: false}];
  ixAccounts.push({pubkey: blogAccount, isSigner: false, isWritable: true});
  if (blogIx.i == 0) {
    ixAccounts.push({pubkey: systemProgramId, isSigner: false, isWritable: true});
  } else {
    ixAccounts.push({pubkey: postAccount, isSigner: false, isWritable: true});
    ixAccounts.push({pubkey: systemProgramId, isSigner: false, isWritable: false});
  }

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
  await viewBlogAccount(connection, blogAccount);
  await viewPostAccount(connection, postAccount);
  // await viewAllAccounts(connection, PROGRAM_ID);
};


/**
  VIEW ACCOUNT DATA ////////////////////////////////////////////////////
 */
// Get Blog Account
async function viewBlogAccount(connection: Connection, account: PublicKey) {

  // Define Blog Account Structure
  const BLOG_ACCOUNT_DATA_LAYOUT = borsh.struct([
    borsh.publicKey("authorityPubkey"),
    borsh.u8("bump"),
    borsh.u8("postCount"),
  ]);

  // Get Blog Account Current State
  const blogAccountInfo = await connection.getAccountInfo(account);

  // Decode and Show Blog Account
  if (blogAccountInfo) { 
    const blogAccountData = BLOG_ACCOUNT_DATA_LAYOUT.decode(blogAccountInfo.data);
    console.log("Blog Account Info: \n", blogAccountData);
  };
};

// View Post Account
async function viewPostAccount(connection: Connection, account: PublicKey) {

  // Define Post Account Structure
  const POST_ACCOUNT_DATA_LAYOUT = borsh.struct([
    borsh.publicKey("author"),
    borsh.publicKey("blog"),
    borsh.u8("bump"),
    borsh.str("slug"),
    borsh.str("title"),
    borsh.str("content"),
  ]);

  // Get Post Account Current Info
  const postAccountInfo = await connection.getAccountInfo(account);

  // Decode and Show Post Account
  if (postAccountInfo) { 
    const postAccountData = POST_ACCOUNT_DATA_LAYOUT.decode(postAccountInfo.data);
    console.log("Post Account Info: \n", postAccountData);
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
