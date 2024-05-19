import * as anchor from '@coral-xyz/anchor';
import { web3 } from '@coral-xyz/anchor';

import { confirmTransaction } from '../util';

export const initializeEnvironment = (PROVIDER: anchor.Provider, user: web3.PublicKey) => {
    it('airdropped user', async () => {
        const airdropUser = await PROVIDER.connection.requestAirdrop(
            user,
            Math.floor(100 * anchor.web3.LAMPORTS_PER_SOL),
        );
        await confirmTransaction(PROVIDER, airdropUser);
    });
};
