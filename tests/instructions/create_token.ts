import * as anchor from '@coral-xyz/anchor';
import { Program } from '@coral-xyz/anchor';
import { web3 } from '@coral-xyz/anchor';

import { Rocket } from '../../target/types/rocket';

import { signAndConfirmTransaction } from '../util';

type CreateTokenParams = {
    name: string;
    symbol: string;
    uri: string;
};

type CreateTokenAccounts = {
    mint: web3.PublicKey;

    bondingCurve: web3.PublicKey;
    associatedBondingCurve: web3.PublicKey;

    metadata: web3.PublicKey;

    user: web3.PublicKey;

    tokenMetadataProgram: web3.PublicKey;
};

export const createToken = async (
    PROVIDER: anchor.Provider,
    PROGRAM: Program<Rocket>,
    params: CreateTokenParams,
    accounts: CreateTokenAccounts,
    signers: anchor.Wallet[],
) => {
    it('allows user to creates a new token', async () => {
        const createIx = await PROGRAM.methods
            .create(params)
            .accountsPartial(accounts)
            .instruction();

        const tx = new anchor.web3.Transaction().add(createIx);

        await signAndConfirmTransaction(PROVIDER, tx, accounts.user, signers);
    });
};
