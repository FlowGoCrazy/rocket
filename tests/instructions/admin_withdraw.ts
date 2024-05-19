import { createAssociatedTokenAccountInstruction } from '@solana/spl-token';
import * as anchor from '@coral-xyz/anchor';
import { Program } from '@coral-xyz/anchor';
import { web3 } from '@coral-xyz/anchor';

import { Rocket } from '../../target/types/rocket';

import { confirmTransaction } from '../util';

type AdminWithdrawAccounts = {
    mint: web3.PublicKey;

    associatedBondingCurve: web3.PublicKey;

    associatedAdmin: web3.PublicKey;
};

export const adminWithdraw = async (
    PROVIDER: anchor.Provider,
    PROGRAM: Program<Rocket>,
    accounts: AdminWithdrawAccounts,
) => {
    it('allows admin to withdrew liquidity', async () => {
        const sig = await PROGRAM.methods.adminWithdraw().accounts(accounts).rpc();
        await confirmTransaction(PROVIDER, sig);
    });
};
