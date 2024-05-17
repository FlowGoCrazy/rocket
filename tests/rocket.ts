import { getAssociatedTokenAddress } from '@solana/spl-token';
import * as anchor from '@coral-xyz/anchor';
import { Program } from '@coral-xyz/anchor';

import { Rocket } from '../target/types/rocket';

import * as dotenv from 'dotenv';
dotenv.config();

const loadPrivateKey = () => {
    const privateKey = process.env.PRIVATE_KEY;
    if (!privateKey) {
        throw new Error('PRIVATE_KEY is not set');
    }
    return new Uint8Array(privateKey.split(', ').map((s) => parseInt(s, 10)));
};

describe('rocket', () => {
    anchor.setProvider(anchor.AnchorProvider.env());
    const provider = anchor.getProvider();

    const program = anchor.workspace.Rocket as Program<Rocket>;

    const keypair = anchor.web3.Keypair.fromSecretKey(loadPrivateKey());
    const wallet = new anchor.Wallet(keypair);

    it('creates a new token', async () => {
        /* generate new addresses */
        const mintKeypair = anchor.web3.Keypair.generate();
        const mintWallet = new anchor.Wallet(mintKeypair);

        const mintAuthorityKeypair = anchor.web3.Keypair.generate();

        const [bondingCurveAddress] = anchor.web3.PublicKey.findProgramAddressSync([mintKeypair.publicKey.toBuffer(), Buffer.from('bonding_curve')], new anchor.web3.PublicKey(program.idl.address));

        const associatedBondingCurve = await getAssociatedTokenAddress(mintKeypair.publicKey, bondingCurveAddress, true);

        const createIx = await program.methods
            .create()
            .accountsPartial({
                mint: mintKeypair.publicKey,
                mintAuthority: mintAuthorityKeypair.publicKey,

                bondingCurve: bondingCurveAddress,
                associatedBondingCurve: associatedBondingCurve,

                signer: wallet.publicKey,
            })
            .instruction();

        const tx = new anchor.web3.Transaction();
        tx.add(createIx);

        // const buyIx = await program.methods
        //     .buy()
        //     .accounts({
        //         mint: mintKeypair.publicKey,
        //         signer: wallet.publicKey,
        //     })
        //     .instruction();
        // tx.add(buyIx);

        /* set blockhash / fee payer */
        const { blockhash, lastValidBlockHeight } = await provider.connection.getLatestBlockhash();
        tx.recentBlockhash = blockhash;
        tx.lastValidBlockHeight = lastValidBlockHeight;
        tx.feePayer = keypair.publicKey;

        /* sign tx with all signer accounts */
        const payerSignedTx = await wallet.signTransaction(tx);
        const mintSignedTx = await mintWallet.signTransaction(payerSignedTx);

        /* send and confirm tx */
        const sig = await anchor.getProvider().connection.sendRawTransaction(mintSignedTx.serialize());
        await anchor.getProvider().connection.confirmTransaction(sig, 'confirmed');

        /* get confirmed tx result */
        const txData = await anchor.getProvider().connection.getParsedTransaction(sig, 'confirmed');
        console.log(txData.meta.logMessages);
    });
});
