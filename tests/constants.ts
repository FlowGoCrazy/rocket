import * as anchor from '@coral-xyz/anchor';
import { Keypair } from '@solana/web3.js';
import bs58 from 'bs58';

import * as dotenv from 'dotenv';
dotenv.config();

const loadPrivateKey = (base58PrivateKey: string) => {
    if (!base58PrivateKey) {
        throw new Error('env var for pk is not set');
    }
    const privateKeyUint8Array = bs58.decode(base58PrivateKey);
    return Keypair.fromSecretKey(privateKeyUint8Array);
};

export const TOKEN_METADATA_PROGRAM_ID = new anchor.web3.PublicKey(
    'metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s',
);

export const PROVIDER_KEYPAIR = loadPrivateKey(process.env.PROVIDER_PRIVATE_KEY);

export const ADMIN = PROVIDER_KEYPAIR.publicKey;
