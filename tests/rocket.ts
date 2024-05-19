import {
    getAssociatedTokenAddress,
    createAssociatedTokenAccountInstruction,
} from '@solana/spl-token';
import * as anchor from '@coral-xyz/anchor';
import { Program } from '@coral-xyz/anchor';

import { Rocket } from '../target/types/rocket';

import * as dotenv from 'dotenv';
import { expect } from 'chai';
import { BN } from 'bn.js';
dotenv.config();

const loadPrivateKey = () => {
    const privateKey = process.env.PRIVATE_KEY;
    if (!privateKey) {
        throw new Error('PRIVATE_KEY is not set');
    }
    return new Uint8Array(privateKey.split(', ').map((s) => parseInt(s, 10)));
};

const TOKEN_METADATA_PROGRAM_ID = new anchor.web3.PublicKey(
    'metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s',
);

describe('rocket', () => {
    anchor.setProvider(anchor.AnchorProvider.env());
    const provider = anchor.getProvider();

    const program = anchor.workspace.Rocket as Program<Rocket>;

    const keypair = anchor.web3.Keypair.fromSecretKey(loadPrivateKey());
    const wallet = new anchor.Wallet(keypair);

    it('can create a new token', async () => {
        /* generate new addresses */
        const mintKeypair = anchor.web3.Keypair.generate();
        const mintWallet = new anchor.Wallet(mintKeypair);

        const [bondingCurveAddress] = anchor.web3.PublicKey.findProgramAddressSync(
            [mintKeypair.publicKey.toBuffer(), Buffer.from('bonding_curve')],
            new anchor.web3.PublicKey(program.idl.address),
        );

        const [metadataAddress] = anchor.web3.PublicKey.findProgramAddressSync(
            [
                Buffer.from('metadata'),
                TOKEN_METADATA_PROGRAM_ID.toBuffer(),
                mintKeypair.publicKey.toBuffer(),
            ],
            TOKEN_METADATA_PROGRAM_ID,
        );

        const associatedBondingCurve = await getAssociatedTokenAddress(
            mintKeypair.publicKey,
            bondingCurveAddress,
            true,
        );

        const associatedUser = await getAssociatedTokenAddress(
            mintKeypair.publicKey,
            wallet.publicKey,
            false,
        );

        const mockReferrerKeypair = anchor.web3.Keypair.generate();

        const tx = new anchor.web3.Transaction();

        const adminUpdateGlobalIx = await program.methods
            .adminUpdateGlobal({
                feeRecipient: wallet.publicKey,
                feeBasisPoints: new BN(100),
                refShareBasisPoints: new BN(25),
                initialVirtualTokenReserves: new BN(1_073_000_000_000_000),
                initialVirtualSolReserves: new BN(30_000_000_000),
                initialRealTokenReserves: new BN(793_100_000_000_000),
                tokenTotalSupply: new BN(1_000_000_000_000_000),
            })
            .instruction();
        tx.add(adminUpdateGlobalIx);

        const createIx = await program.methods
            .create({
                name: 'Test Rocket Token',
                symbol: 'TRT',
                uri: 'https://cf-ipfs.com/ipfs/QmSaKVNYHCc4cRU4Wks8nbYqpUr3ZpGdTi7mRdmcrXD9h6',
            })
            .accountsPartial({
                mint: mintKeypair.publicKey,

                bondingCurve: bondingCurveAddress,
                associatedBondingCurve: associatedBondingCurve,

                metadata: metadataAddress,

                user: wallet.publicKey,

                tokenMetadataProgram: TOKEN_METADATA_PROGRAM_ID,
            })
            .instruction();
        tx.add(createIx);

        const createAtaIx = await createAssociatedTokenAccountInstruction(
            wallet.publicKey,
            associatedUser,
            wallet.publicKey,
            mintKeypair.publicKey,
        );
        tx.add(createAtaIx);

        // const swapFixedSolToTokenIx = await program.methods
        //     .swapFixedSolToToken(new BN(1_000_000_000), new BN(34_612_903_225_806))
        //     .accounts({
        //         mint: mintKeypair.publicKey,

        //         associatedBondingCurve: associatedBondingCurve,

        //         user: wallet.publicKey,
        //         associatedUser: associatedUser,
        //     })
        //     .instruction();
        // tx.add(swapFixedSolToTokenIx);

        const swapSolToFixedTokenIx = await program.methods
            // .swapSolToFixedToken(new BN(50_000_000_000_000), new BN(1_466_275_659))
            .swapSolToFixedToken(new BN(793_100_000_000_000), new BN(86_000_000_000))
            .accounts({
                feeRecipient: wallet.publicKey,
                referrer: mockReferrerKeypair.publicKey,

                mint: mintKeypair.publicKey,

                associatedBondingCurve: associatedBondingCurve,

                user: wallet.publicKey,
                associatedUser: associatedUser,
            })
            .instruction();
        tx.add(swapSolToFixedTokenIx);

        // const swapFixedTokenToSolIx = await program.methods
        //     .swapFixedTokenToSol(new BN(50_000_000_000_000), new BN(1_000_000_000))
        //     .accounts({
        //         feeRecipient: wallet.publicKey,
        //         referrer: mockReferrerKeypair.publicKey,

        //         mint: mintKeypair.publicKey,

        //         associatedBondingCurve: associatedBondingCurve,

        //         user: wallet.publicKey,
        //         associatedUser: associatedUser,
        //     })
        //     .instruction();
        // tx.add(swapFixedTokenToSolIx);

        const adminWithdrawIx = await program.methods
            .adminWithdraw()
            .accounts({
                mint: mintKeypair.publicKey,

                associatedBondingCurve: associatedBondingCurve,

                associatedAdmin: associatedUser,
            })
            .instruction();
        tx.add(adminWithdrawIx);

        /* set blockhash / fee payer */
        const { blockhash, lastValidBlockHeight } = await provider.connection.getLatestBlockhash();
        tx.recentBlockhash = blockhash;
        tx.lastValidBlockHeight = lastValidBlockHeight;
        tx.feePayer = keypair.publicKey;

        /* sign tx with all signer accounts */
        const payerSignedTx = await wallet.signTransaction(tx);
        const mintSignedTx = await mintWallet.signTransaction(payerSignedTx);

        /* send and confirm tx */
        const sig = await anchor
            .getProvider()
            .connection.sendRawTransaction(mintSignedTx.serialize(), {
                skipPreflight: true,
            });
        await anchor.getProvider().connection.confirmTransaction(sig, 'confirmed');

        /* get confirmed tx result */
        const txData = await anchor.getProvider().connection.getParsedTransaction(sig, 'confirmed');
        console.log(JSON.stringify(txData.meta.logMessages, null, 2));

        expect(txData.meta.err).to.eq(null);
    });
});
