import * as anchor from '@project-serum/anchor';
import web3, { PublicKey, SYSVAR_RENT_PUBKEY } from '@solana/web3.js';
import BN from 'bn.js';
import { TOKEN_PROGRAM_ID, createAccount, ASSOCIATED_TOKEN_PROGRAM_ID, getAccount } from "@solana/spl-token";

const idl = JSON.parse(
    require("fs").readFileSync("target/idl/test.json", "utf8")
);

// Address of the deployed program.
const programId = new anchor.web3.PublicKey("EcDwM6SLq81xpKS1ykf7UGjjyE84KJvjmAzWmLwy9tJx");

// Generate the program client from IDL.
const program = new anchor.Program(idl, programId, anchor.Provider.env());

function getCLIKeypair(): anchor.web3.Keypair {
    // // main
    // let rawKey = [3,51,234,104,214,237,240,135,174,139,47,56,7,163,19,2,185,101,172,186,186,148,81,223,112,107,112,201,41,238,115,176,107,186,85,199,132,199,69,249,72,231,167,1,217,206,30,249,93,57,83,226,51,136,9,104,153,90,240,100,62,157,10,250];
    // rawKey = rawKey.slice(0, 32);
    // return anchor.web3.Keypair.fromSeed(Uint8Array.from(rawKey));    
    
    let rawKey = [23,16,124,22,170,23,145,113,158,37,54,139,135,12,247,9,31,90,120,224,69,190,29,165,229,157,91,48,225,101,236,206,78,75,26,57,69,192,128,134,190,88,33,226,76,22,108,187,21,118,195,53,29,49,207,84,169,6,212,75,73,14,120,212];
    rawKey = rawKey.slice(0, 32);
    return anchor.web3.Keypair.fromSeed(Uint8Array.from(rawKey));    
}

const setNewField = async function() {
    const keypair = getCLIKeypair();

    const [networkCfg] = await PublicKey.findProgramAddress([Buffer.from("config")], programId);
    const txID = await program.rpc.setNewField(new BN("60"), {
        accounts: {
            networkConfig: networkCfg,
            
        },
        signers: [
            keypair,
        ],
    });
    console.log(txID.toString());
}

async function initpoolv2() {
    const keypair = getCLIKeypair();
    const usdcMint = new anchor.web3.PublicKey("BM82b8KV4pdgEo2we57myt7Du9zFaNPt5UC4F9oYsjpy");
    const [pool] = await PublicKey.findProgramAddress([Buffer.from("pool_v4")], programId);
    const txID = await program.rpc.initPoolV2({
        accounts: {
            payer: program.provider.wallet.publicKey,
            systemProgram: anchor.web3.SystemProgram.programId,
            usdcMint: usdcMint,
            tokenProgram: TOKEN_PROGRAM_ID,
            rent: SYSVAR_RENT_PUBKEY,
            poolAccount: pool,
        },
        signers: [
            keypair,
        ],
    });
    console.log(txID.toString());
}

const main = async function() {
    const keypair = getCLIKeypair();

    // Execute the RPC.
    const purchaseProtectionFeeBasisPoints = new BN("300");
    const merchantTxFeeBasisPoints = new BN("100");

    const shekelTokenMint = new anchor.web3.PublicKey("3kfCx9Bz4gccL9pcpp7D3a13KjpFaCuv7NirfetjVkit");
    const usdcMint = new anchor.web3.PublicKey("BM82b8KV4pdgEo2we57myt7Du9zFaNPt5UC4F9oYsjpy");

    const [networkCfg] = await PublicKey.findProgramAddress([Buffer.from("config_v4")], programId);
    const [stats] = await PublicKey.findProgramAddress([Buffer.from("stats_v4")], programId);
    const [treasury] = await PublicKey.findProgramAddress([Buffer.from("treasury_v4")], programId);
    const [pool] = await PublicKey.findProgramAddress([Buffer.from("pool_v4")], programId);
    const [authority] = await PublicKey.findProgramAddress([Buffer.from("authority_v4")], programId);

    const txID = await program.rpc.init(merchantTxFeeBasisPoints, purchaseProtectionFeeBasisPoints, {
        accounts: {
            networkConfig: networkCfg,
            payer: program.provider.wallet.publicKey,
            systemProgram: anchor.web3.SystemProgram.programId,
            tokenTreasury: treasury,
            stats: stats,
            shekelTokenMint: shekelTokenMint,
            usdcMint: usdcMint,
            tokenProgram: TOKEN_PROGRAM_ID,
            rent: SYSVAR_RENT_PUBKEY,
            poolAccount: pool,
            authority: authority,
        },
        signers: [
            keypair,
        ],
    });
    console.log(txID.toString());
};

const transact = async function() {
    const keypair = getCLIKeypair();    

    // Execute the RPC.
    const [networkCfg] = await PublicKey.findProgramAddress([Buffer.from("config_v4")], programId);
    const [stats] = await PublicKey.findProgramAddress([Buffer.from("stats_v4")], programId);
    const [treasury] = await PublicKey.findProgramAddress([Buffer.from("treasury_v4")], programId);
    const [pool] = await PublicKey.findProgramAddress([Buffer.from("pool_v4")], programId);
    const [authority] = await PublicKey.findProgramAddress([Buffer.from("authority_v4")], programId);

    const txID = await program.rpc.transact(new BN("1000000"), {
        accounts: {
            owner: keypair.publicKey,
            destination: new anchor.web3.PublicKey("PB8zR3iZ3SyQpi1juRrVhKXPNoRgT3H1QFGTSpPjJmE"),
            destinationTokenAccount: new anchor.web3.PublicKey("HJsWAkNjcHSgGPX1Rx2XwCAow9EFYfL25RECow8V6SVP"),
            source: new anchor.web3.PublicKey("7XH2YDxfmnpBMQcp1nHH9v9gcJQNBHd8N3rNmJYeB6i6"),
            sourceTokenAccount: new anchor.web3.PublicKey("98zabcFKTiD9DwdgtQumQPk4PWMkeVVhKrHVJUz5J3fH"),

            splTokenProgram: TOKEN_PROGRAM_ID,
            networkConfig: networkCfg,
            poolAccount: pool,
            stats: stats,
            treasury: treasury,
            authority: authority,
        },
        signers: [
            keypair,
        ],
    });
    console.log(txID.toString());
}

async function get() {
    const conn = new anchor.web3.Connection("https://api.google.devnet.solana.com");
    const [treasury] = await PublicKey.findProgramAddress([Buffer.from("treasury_v4")], programId);
    const data = await getAccount(conn, treasury);
    console.log(data.mint.toString());
}

async function get_ata() {
    const tokenAddress = await findAssociatedTokenAddress(
        new PublicKey('8FXRKgS2nDJ1axRRTvdgkQudUsBZZ5gKnp4zF1kK6vMw'), 
        new PublicKey('BM82b8KV4pdgEo2we57myt7Du9zFaNPt5UC4F9oYsjpy'),
    );
    console.log(tokenAddress.toString());
}

async function createSpl() {
    const forAcc = new anchor.web3.PublicKey("44DHMG3SzMRbxnLvoqvrn4YZ689xC5XWc5wt8pPv3QkA");
    const conn = new anchor.web3.Connection("https://api.google.devnet.solana.com");
    const mint = new anchor.web3.PublicKey("AUPVPPeVQPdFyYQdtzxPYGcmjPxEWgy92wYEkAeJuh8o");

    let rawKey = [3,51,234,104,214,237,240,135,174,139,47,56,7,163,19,2,185,101,172,186,186,148,81,223,112,107,112,201,41,238,115,176,107,186,85,199,132,199,69,249,72,231,167,1,217,206,30,249,93,57,83,226,51,136,9,104,153,90,240,100,62,157,10,250];
    rawKey = rawKey.slice(0, 32);
    const keypair = anchor.web3.Keypair.fromSeed(Uint8Array.from(rawKey));

    const data = await createAccount(conn, keypair, mint, forAcc);
    console.log(data.toString());
}

async function print() {
    const [networkCfg] = await PublicKey.findProgramAddress([Buffer.from("config_v4")], programId);
    const [stats] = await PublicKey.findProgramAddress([Buffer.from("stats_v4")], programId);
    const [treasury] = await PublicKey.findProgramAddress([Buffer.from("treasury_v4")], programId);
    const [pool] = await PublicKey.findProgramAddress([Buffer.from("pool_v4")], programId);
    const [authority] = await PublicKey.findProgramAddress([Buffer.from("authority_v4")], programId);
    console.log("pool", pool.toString());
    console.log("treasury", treasury.toString());
    console.log("stats", stats.toString());
    console.log("config", networkCfg.toString());
    console.log("authority", authority.toString());
}

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

transact();

