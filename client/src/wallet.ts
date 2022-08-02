import { Keypair, Connection, LAMPORTS_PER_SOL } from "@solana/web3.js";
import * as bip39 from "bip39";
import { SELECTED_RPC_URL } from "./utils";
import * as fs from "fs";

// https://medium.com/@lianxiongdi/solana-development-1-basic-operation-of-solana-cli-dcf156137e6

// Connect
const connection = new Connection(SELECTED_RPC_URL, "confirmed");

// Run Commands
async function main() {
  const arg = process.argv.slice(2).toString();
  if (!arg) {return console.log("No argument specified")};
  await createWallet(arg);
}

// Create Wallet
async function createWallet(walletLabel: string) {

  let mnemonic = bip39.generateMnemonic();
  const seed = bip39.mnemonicToSeedSync(mnemonic, ""); // (mnemonic, password)
  const keypair = Keypair.fromSeed(seed.slice(0, 32));

  // Sign Airdrop Tokens Request
  const airdropSignature = await connection.requestAirdrop(
    keypair.publicKey,
    LAMPORTS_PER_SOL * 2
  );
  await connection.confirmTransaction(airdropSignature);

  // Write Secret Key
  fs.writeFileSync(`./src/keys/${walletLabel}_pubkey.json`, JSON.stringify(keypair.publicKey.toBase58()));
  fs.writeFileSync(`./src/keys/${walletLabel}_seckey.json`, JSON.stringify(keypair.secretKey.toString()));
  
  // Return Results
  console.log("PublicKey: ", keypair.publicKey.toBase58());
  console.log("Balance SOL: ", await connection.getBalance(keypair.publicKey));
  console.log(walletLabel + " wallet successfully created.");
}

// Run Setup
main();
