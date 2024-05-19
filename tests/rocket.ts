import * as anchor from '@coral-xyz/anchor';
import { Program } from '@coral-xyz/anchor';

import { Rocket } from '../target/types/rocket';

import {
    initializeEnvironment,
    updateGlobal,
    createToken,
    swapSolToFixedToken,
    adminWithdraw,
} from './instructions';
import {
    generateMintAccounts,
    generateBondingCurveAccounts,
    generateUserAccounts,
    generateReferrerAccounts,
    generateFeeRecipientAccount,
    generateAdminAccounts,
} from './util';

import { ADMIN, TOKEN_METADATA_PROGRAM_ID } from './constants';

describe('rocket', async () => {
    anchor.setProvider(anchor.AnchorProvider.env());

    const PROVIDER = anchor.getProvider();
    const PROGRAM = anchor.workspace.Rocket as Program<Rocket>;

    const { mintKeypair, mintWallet, metadataPDA } = await generateMintAccounts(PROGRAM);
    const { adminATA } = await generateAdminAccounts(mintKeypair.publicKey, ADMIN);
    const { userKeypair, userWallet, userATA } = await generateUserAccounts(
        PROGRAM,
        mintKeypair.publicKey,
    );
    const { bondingCurvePDA, bondingCurveATA } = await generateBondingCurveAccounts(
        PROGRAM,
        mintKeypair.publicKey,
    );
    const { referrer, referrerPDA } = await generateReferrerAccounts(PROGRAM);
    const { feeRecipient } = await generateFeeRecipientAccount();

    initializeEnvironment(PROVIDER, userKeypair.publicKey);

    updateGlobal(PROVIDER, PROGRAM, {
        feeRecipient: feeRecipient,
        feeBasisPoints: new anchor.BN(100),
        refShareBasisPoints: new anchor.BN(25),
        initialVirtualTokenReserves: new anchor.BN(1_073_000_000_000_000),
        initialVirtualSolReserves: new anchor.BN(30_000_000_000),
        initialRealTokenReserves: new anchor.BN(793_100_000_000_000),
        tokenTotalSupply: new anchor.BN(1_000_000_000_000_000),
    });

    createToken(
        PROVIDER,
        PROGRAM,
        {
            name: 'Test Rocket Token',
            symbol: 'TRT',
            uri: 'https://cf-ipfs.com/ipfs/QmSaKVNYHCc4cRU4Wks8nbYqpUr3ZpGdTi7mRdmcrXD9h6',
        },
        {
            mint: mintKeypair.publicKey,

            bondingCurve: bondingCurvePDA,
            associatedBondingCurve: bondingCurveATA,

            metadata: metadataPDA,

            user: userKeypair.publicKey,

            tokenMetadataProgram: TOKEN_METADATA_PROGRAM_ID,
        },
        [userWallet, mintWallet],
    );

    swapSolToFixedToken(
        PROVIDER,
        PROGRAM,
        [new anchor.BN(793_100_000_000_000), new anchor.BN(86_000_000_000)] /* buy entire curve */,
        {
            feeRecipient: feeRecipient,
            referrer: referrer,

            mint: mintKeypair.publicKey,

            associatedBondingCurve: bondingCurveATA,

            user: userKeypair.publicKey,
            associatedUser: userATA,
        },
        [userWallet],
    );

    adminWithdraw(PROVIDER, PROGRAM, {
        mint: mintKeypair.publicKey,

        associatedBondingCurve: bondingCurveATA,

        associatedAdmin: adminATA,
    });

    // it('checks balances', async () => {
    //     const feeRecipientBalance = await PROVIDER.connection.getBalance(feeRecipient);
    //     console.log('fee recipient balance:', feeRecipientBalance);

    //     const referrerBalance = await PROVIDER.connection.getBalance(referrerPDA);
    //     console.log('referrer balance:', referrerBalance);

    //     /* get balance of referrer pda */
    //     const referrerAccount = await PROGRAM.account.userRef.fetch(referrerPDA, 'processed');
    //     console.log(referrerAccount.balance.toString());
    // });
});
