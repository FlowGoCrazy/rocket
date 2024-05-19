import { createAssociatedTokenAccountInstruction } from '@solana/spl-token';
import * as anchor from '@coral-xyz/anchor';
import { Program } from '@coral-xyz/anchor';
import { web3 } from '@coral-xyz/anchor';

import { Rocket } from '../../target/types/rocket';

import { signAndConfirmTransaction } from '../util';

type SwapSolToFixedTokenParams = [tokensOut: anchor.BN, maxSolIn: anchor.BN];

export type SwapAccounts = {
    feeRecipient: web3.PublicKey;
    referrer: web3.PublicKey;

    mint: web3.PublicKey;

    associatedBondingCurve: web3.PublicKey;

    user: web3.PublicKey;
    associatedUser: web3.PublicKey;
};

export const swapSolToFixedToken = (
    PROVIDER: anchor.Provider,
    PROGRAM: Program<Rocket>,
    params: SwapSolToFixedTokenParams,
    accounts: SwapAccounts,
    signers: anchor.Wallet[],
) => {
    it('allows user to swap fixed sol amount to tokens', async () => {
        const tx = new anchor.web3.Transaction();

        const createUserAtaIx = await createAssociatedTokenAccountInstruction(
            accounts.user,
            accounts.associatedUser,
            accounts.user,
            accounts.mint,
        );
        tx.add(createUserAtaIx);

        const initUserRefIx = await PROGRAM.methods
            .initUserRef()
            .accounts({
                user: accounts.referrer,
                signer: accounts.user,
            })
            .instruction();
        tx.add(initUserRefIx);

        const swapSolToFixedTokenIx = await PROGRAM.methods
            .swapSolToFixedToken(...params)
            .accounts(accounts)
            .instruction();
        tx.add(swapSolToFixedTokenIx);

        await signAndConfirmTransaction(PROVIDER, tx, accounts.user, signers);
    });
};
