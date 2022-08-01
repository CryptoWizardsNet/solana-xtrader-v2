import { Keypair, PublicKey, Connection, LAMPORTS_PER_SOL } from "@solana/web3.js";
import * as bip39 from "bip39";
import { SELECTED_RPC_URL } from "./utils";
import * as fs from "fs";

// https://medium.com/@lianxiongdi/solana-development-1-basic-operation-of-solana-cli-dcf156137e6

// Connect
const connection = new Connection(SELECTED_RPC_URL, "confirmed");

// Run Commands
async function main() {
  const arg = process.argv.slice(2).toString();
  await createWallet(arg);
  console.log(arg + " wallet successfully created.")
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

  return keypair.publicKey;
}

// Run Setup
main();
