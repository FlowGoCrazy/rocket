import * as anchor from '@coral-xyz/anchor';
import { Program } from '@coral-xyz/anchor';
import { Rocket } from '../target/types/rocket';

describe('rocket', () => {
    anchor.setProvider(anchor.AnchorProvider.env());

    const program = anchor.workspace.Rocket as Program<Rocket>;

    it('initializes', async () => {
        const sig = await program.methods.initialize().rpc();
        console.log('init sig:', sig);

        await anchor.getProvider().connection.confirmTransaction(sig, 'confirmed');

        const txData = await anchor.getProvider().connection.getParsedTransaction(sig, 'confirmed');
        console.log(txData.meta.logMessages);
    });
});
