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


/**
    CONNECTION AND PAYER ////////////////////////////////////////////////
**/


// Connect to Solana and get Payer
const RPC_URL = "http://127.0.0.1:8899"
let connection: Connection;
let payer: Keypair;
async function connectPayer() {
  connection = new Connection(RPC_URL, "confirmed");
  payer = await getPayer();
}


/**
    PROGRAM ID //////////////////////////////////////////////////////////
**/


// Define Program Id
let programId: PublicKey;
async function getProgramPublicKey(): Promise<void> {
  const PROGRAM_PATH = path.resolve(__dirname, '../../program/target/deploy/');
  const PROGRAM_KEYPAIR_PATH = path.join(PROGRAM_PATH, 'solana_boilerplate-keypair.json');
  const programKeypair = await createKeypairFromFile(PROGRAM_KEYPAIR_PATH);
  programId = programKeypair.publicKey;
}


/**
    ACCOUNT AND INSTRUCTION SCHEMA //////////////////////////////////////////////////
**/


// ACCOUNT CLASS
class Account {
  counter = 0;
  constructor(fields: {counter: number} | undefined = undefined) {
    if (fields) {
      this.counter = fields.counter;
    }
  }
}

// INSTRUCTION SCHEMA
const AccountSchema = new Map([
  [Account, {kind: 'struct', fields: [['counter', 'u8']]}],
]);

// ACCOUNT DATA
const account_data = serialize(
  AccountSchema,
  new Account(),
);


// INSTRUCTION CLASS
class Instruction {
  tag = 0;
  some_data = 10;
  constructor(fields: {tag: number, some_data: number} | undefined = undefined) {
    if (fields) {
      this.tag = fields.tag;
      this.some_data = fields.some_data;
    }
  }
}

// INSTRUCTION SCHEMA
const InstructionSchema = new Map([
  [Instruction, {kind: 'struct', fields: [['tag', 'u8'], ['some_data', 'u8']]}],
]);

// INSTRUCTION BUFFER BYTE ARRAY
const instruction_data = Buffer.from(serialize(
  InstructionSchema,
  new Instruction(),
));

// INSTRUCTION CHECK
// const instruction_data = Buffer.from(new Uint8Array([0, 10]));
console.log("Serialized Instruction: ", instruction_data);


/**
    ACCOUNT MANAGEMENT //////////////////////////////////////////////////
**/


// Structure and Add Accounts to Solana
let callPubkey: PublicKey;
async function accountManagement(): Promise<void> {
  // Note, if this is rejected, it might be because you need to remove the key's saved
  const arr1Length = account_data.length; // Number of bytes in Account1

  // Derive the address (public key) of a greeting account from the program so that it's easy to find later.
  const KEY_SEED = "my test key seed";
  callPubkey = await PublicKey.createWithSeed(
    payer.publicKey,
    KEY_SEED,
    programId,
  );

  // Check if account already exists and create account if not
  const betAccount = await connection.getAccountInfo(callPubkey);
  if (betAccount === null) {

    // Check Lamports required for creating account
    const lamports = await connection.getMinimumBalanceForRentExemption(arr1Length);

    // Build Transaction to Create Account
    const transaction = new Transaction().add(
      SystemProgram.createAccountWithSeed({
        fromPubkey: payer.publicKey,
        basePubkey: payer.publicKey,
        seed: KEY_SEED,
        newAccountPubkey: callPubkey,
        lamports,
        space: arr1Length,
        programId,
      }),
    );

    // Send Transaction for Creating Account
    await sendAndConfirmTransaction(connection, transaction, [payer]);
  }
}


/**
    CALL PROGRAM //////////////////////////////////////////////////
**/


// Place your Bitcoin Bet
export async function placeCall(): Promise<void> {
  console.log('Placing call for Account: ', callPubkey.toBase58());
  console.log('Account signed by: ', payer.publicKey.toBase58());
  console.log('Program Id: ', programId.toBase58());
  const instruction = new TransactionInstruction({
    keys: [{pubkey: callPubkey, isSigner: false, isWritable: true}],
    programId,
    data: instruction_data,
  });
  await sendAndConfirmTransaction(
    connection,
    new Transaction().add(instruction),
    [payer],
  );
}

// View Results
export async function reportResults(): Promise<void> {
  const accountInfo = await connection.getAccountInfo(callPubkey);
  if (accountInfo === null) {
    throw "Error: cannot find the greeted account";
  }
  // Convert to number
  // const view = new Int8Array(accountInfo.data);
  // console.log(view);
  const account = deserialize(
    AccountSchema,
    Account,
    accountInfo.data,
  );
  console.log("Account Updated to: ", account);
}

// Main Function Calls
async function main() {
  await connectPayer();
  await getProgramPublicKey();
  await accountManagement();
  await placeCall();
  await reportResults();
}

// Call Main
main();
