/* eslint-disable @typescript-eslint/no-unsafe-assignment */
/* eslint-disable @typescript-eslint/no-unsafe-member-access */
/* eslint-disable @typescript-eslint/no-unsafe-return */

import fs from 'mz/fs';
import {Keypair, PublicKey} from '@solana/web3.js';
import path from 'path';

const getSecretKey = (label: string) =>
  JSON.parse(fs.readFileSync(`./src/keys/${label}_seckey.json`) as unknown as string);

export async function getKeypair(label: string) {
  const secretKey = getSecretKey(label);
  const secretKeyArray = Uint8Array.from(secretKey.split(","));
  const keypair = Keypair.fromSecretKey(secretKeyArray);
  return keypair;
}

export async function createKeypairFromFile(
  filePath: string,
): Promise<Keypair> {
  const secretKeyString = await fs.readFile(filePath, {encoding: 'utf8'});
  const secretKey = Uint8Array.from(JSON.parse(secretKeyString));
  return Keypair.fromSecretKey(secretKey);
}

export async function getProgramId(): Promise<PublicKey> {
  const PROGRAM_PATH = path.resolve(__dirname, '../../program/target/deploy/');
  const PROGRAM_KEYPAIR_PATH = path.join(PROGRAM_PATH, 'trade-keypair.json');
  const programKeypair = await createKeypairFromFile(PROGRAM_KEYPAIR_PATH);
  const PROGRAM_ID: PublicKey = programKeypair.publicKey;
  return PROGRAM_ID;
}

// Chainlink Price Feed - Devnet
const CHAINLINK_PRICE_FEED = "HgTtcbcmp5BeThax5AU8vg4VwK79qAvAKKFMs8txMLW6"; // Devnet
export const accountChainlinkPriceFeed = new PublicKey(CHAINLINK_PRICE_FEED);

// Chainlink Program Owner - Devnet
const CHAINLINK_ONCHAIN_PROGRAM_OWNER = "HEvSKofvBgfaexv23kMabbYqxasxU3mQ4ibBMEmJWHny";
export const accountChainlinkProgramOwner = new PublicKey(CHAINLINK_ONCHAIN_PROGRAM_OWNER);

// RPC Urls
const DEV_RPC_URL = "https://api.devnet.solana.com";
const LOCAL_RPC_URL = "http://127.0.0.1:8899";
export const SELECTED_RPC_URL = DEV_RPC_URL;
