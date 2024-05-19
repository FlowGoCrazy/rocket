import { getAssociatedTokenAddress } from '@solana/spl-token';
import * as anchor from '@coral-xyz/anchor';
import { Program } from '@coral-xyz/anchor';
import { web3 } from '@coral-xyz/anchor';

import { Rocket } from '../target/types/rocket';

import { TOKEN_METADATA_PROGRAM_ID } from './constants';

export const generateMintAccounts = async (PROGRAM: Program<Rocket>) => {
    const mintKeypair = anchor.web3.Keypair.generate();
    const mintWallet = new anchor.Wallet(mintKeypair);

    const [metadataPDA] = anchor.web3.PublicKey.findProgramAddressSync(
        [
            Buffer.from('metadata'),
            TOKEN_METADATA_PROGRAM_ID.toBuffer(),
            mintKeypair.publicKey.toBuffer(),
        ],
        TOKEN_METADATA_PROGRAM_ID,
    );

    return { mintKeypair, mintWallet, metadataPDA };
};

export const generateAdminAccounts = async (mint: web3.PublicKey, admin: web3.PublicKey) => {
    const adminATA = await getAssociatedTokenAddress(mint, admin, false);
    return { adminATA };
};

export const generateUserAccounts = async (PROGRAM: Program<Rocket>, mint: web3.PublicKey) => {
    const userKeypair = anchor.web3.Keypair.generate();
    const userWallet = new anchor.Wallet(userKeypair);

    const userATA = await getAssociatedTokenAddress(mint, userKeypair.publicKey, false);

    return { userKeypair, userWallet, userATA };
};

export const generateBondingCurveAccounts = async (
    PROGRAM: Program<Rocket>,
    mint: web3.PublicKey,
) => {
    const [bondingCurvePDA] = anchor.web3.PublicKey.findProgramAddressSync(
        [mint.toBuffer(), Buffer.from('bonding_curve')],
        new web3.PublicKey(PROGRAM.idl.address),
    );

    const bondingCurveATA = await getAssociatedTokenAddress(mint, bondingCurvePDA, true);

    return { bondingCurvePDA, bondingCurveATA };
};

export const generateFeeRecipientAccount = async () => {
    const feeRecipient = anchor.web3.Keypair.generate().publicKey;
    return { feeRecipient };
};

export const generateReferrerAccounts = async (PROGRAM: Program<Rocket>) => {
    const referrerKeypair = anchor.web3.Keypair.generate();
    const referrer = referrerKeypair.publicKey;

    const [referrerPDA] = anchor.web3.PublicKey.findProgramAddressSync(
        [referrer.toBuffer(), Buffer.from('ref')],
        new anchor.web3.PublicKey(PROGRAM.idl.address),
    );

    return { referrer, referrerPDA };
};

export const confirmTransaction = async (PROVIDER: anchor.Provider, signature: string) => {
    const { blockhash, lastValidBlockHeight } = await PROVIDER.connection.getLatestBlockhash();
    await PROVIDER.connection.confirmTransaction(
        {
            signature,
            blockhash,
            lastValidBlockHeight,
        },
        'confirmed',
    );
};

export const signAndConfirmTransaction = async (
    PROVIDER: anchor.Provider,
    tx: anchor.web3.Transaction,
    feePayer: web3.PublicKey,
    signers: anchor.Wallet[],
) => {
    /* set blockhash / fee payer */
    const { blockhash, lastValidBlockHeight } = await PROVIDER.connection.getLatestBlockhash();
    tx.recentBlockhash = blockhash;
    tx.lastValidBlockHeight = lastValidBlockHeight;
    tx.feePayer = feePayer;

    /* sign tx with all signers */
    let signedTx = tx;
    for (let signer of signers) {
        signedTx = await signer.signTransaction(signedTx);
    }

    /* confirm tx */
    const sig = await PROVIDER.connection.sendRawTransaction(signedTx.serialize(), {
        skipPreflight: true,
    });
    await confirmTransaction(PROVIDER, sig);

    /* get confirmed tx result */
    const txData = await PROVIDER.connection.getParsedTransaction(sig, 'confirmed');

    if (txData.meta.err != null) {
        console.log(JSON.stringify(txData.meta.logMessages, null, 2));
        console.log(txData.meta.err);
        throw txData.meta.err;
    }
};
