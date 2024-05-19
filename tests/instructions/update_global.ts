import * as anchor from '@coral-xyz/anchor';
import { Program } from '@coral-xyz/anchor';
import { web3 } from '@coral-xyz/anchor';

import { Rocket } from '../../target/types/rocket';

import { confirmTransaction } from '../util';

type InitializeEnvironmentParams = {
    feeRecipient: web3.PublicKey;
    feeBasisPoints: anchor.BN;
    refShareBasisPoints: anchor.BN;
    initialVirtualTokenReserves: anchor.BN;
    initialVirtualSolReserves: anchor.BN;
    initialRealTokenReserves: anchor.BN;
    tokenTotalSupply: anchor.BN;
};

export const updateGlobal = (
    PROVIDER: anchor.Provider,
    PROGRAM: Program<Rocket>,
    params: InitializeEnvironmentParams,
) => {
    it('allows admin to update global state', async () => {
        const sig = await PROGRAM.methods.adminUpdateGlobal(params).rpc();
        await confirmTransaction(PROVIDER, sig);
    });
};
