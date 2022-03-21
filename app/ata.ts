import { PublicKey } from '@solana/web3.js';
import { TOKEN_PROGRAM_ID, ASSOCIATED_TOKEN_PROGRAM_ID } from '@solana/spl-token';

async function findAssociatedTokenAddress(
    walletAddress: PublicKey,
    tokenMintAddress: PublicKey
): Promise<PublicKey> {
    return (await PublicKey.findProgramAddress(
        [
            walletAddress.toBuffer(),
            TOKEN_PROGRAM_ID.toBuffer(),
            tokenMintAddress.toBuffer(),
        ],
        ASSOCIATED_TOKEN_PROGRAM_ID
    ))[0];
}

async function main() {
    const tokenAddress = await findAssociatedTokenAddress(
        new PublicKey('8FXRKgS2nDJ1axRRTvdgkQudUsBZZ5gKnp4zF1kK6vMw'), 
        new PublicKey('AUPVPPeVQPdFyYQdtzxPYGcmjPxEWgy92wYEkAeJuh8o'),
    );
    console.log(tokenAddress.toString());
}

main();