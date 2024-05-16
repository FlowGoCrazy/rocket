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

    const bcKeypair = anchor.web3.Keypair.generate();
    const bcWallet = new anchor.Wallet(bcKeypair);

    it('initializes', async () => {
        const createIx = await program.methods
            .create()
            .accounts({
                bondingCurve: bcKeypair.publicKey,
                signer: wallet.publicKey,
            })
            .instruction();

        const tx = new anchor.web3.Transaction();
        tx.add(createIx);

        const buyIx = await program.methods
            .buy()
            .accounts({
                bondingCurve: bcKeypair.publicKey,
                signer: wallet.publicKey,
            })
            .instruction();
        tx.add(buyIx);

        /* set blockhash / fee payer */
        const { blockhash, lastValidBlockHeight } = await provider.connection.getLatestBlockhash();
        tx.recentBlockhash = blockhash;
        tx.lastValidBlockHeight = lastValidBlockHeight;
        tx.feePayer = keypair.publicKey;

        /* sign tx with all signer accounts */
        const payerSignedTx = await wallet.signTransaction(tx);
        const bcSignedTx = await bcWallet.signTransaction(payerSignedTx);

        /* send and confirm tx */
        const sig = await anchor.getProvider().connection.sendRawTransaction(bcSignedTx.serialize());
        await anchor.getProvider().connection.confirmTransaction(sig, 'confirmed');

        /* get confirmed tx result */
        const txData = await anchor.getProvider().connection.getParsedTransaction(sig, 'confirmed');
        console.log(txData.meta.logMessages);
    });
});
