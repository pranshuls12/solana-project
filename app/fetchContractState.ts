import * as Web3 from '@solana/web3.js';
import * as fs from 'fs';
import dotenv from 'dotenv';
import * as anchor from '@project-serum/anchor';
import * as borsh from "@coral-xyz/borsh";
dotenv.config();
import { Program } from '@project-serum/anchor';
import { NeptuneLbp } from "../target/types/neptune_lbp";
import { publicKey } from '@project-serum/anchor/dist/cjs/utils';
import * as schemas from "./borshAccountsSchema"

  
  
// import localwallet from "./wallet.json" assert { type: "json" };

const pg = {
    // wallet: { keypair: initializeKeypair( new Web3.Connection( Web3.clusterApiUrl('devnet') ) ) },
    connection: new Web3.Connection( "http://127.0.0.1:8899" )
};


// Configure the client to use the local cluster.
const programId = new Web3.PublicKey("7iHrHPTAuR1bfnXeVRLBuDrAWVDEMFHTww2VrL2tJuKS")


export async function fetchContractState(connection: Web3.Connection){
    const config : Web3.GetProgramAccountsConfig = {
        dataSlice: { offset: 0, length: 0 }
    }
    const accounts = await connection.getProgramAccounts(programId,config)
    accounts.forEach(({pubkey,account}) => {
        console.log('Account:', pubkey.toBase58())
        console.log('DataBuffer',account.data)
    });
    const accountKeys = accounts.map(account => account.pubkey)
    const paginatedKeys = accountKeys.slice(0, 10)
    const accountInfos = await connection.getMultipleAccountsInfo(paginatedKeys)

    accountInfos.forEach(accountInfo => {
        //console.log(accountInfo.data)
         // Désérialisez d'abord la clé de type de compte
        const { account_type } = schemas.keySchema.decode(accountInfo.data);
        console.log(account_type)
        // Choisissez le schéma de désérialisation basé sur la clé
        let decodedData;
        // Utiliser la clé pour choisir le bon schéma de désérialisation
  switch(account_type) {
    case 1: // Type pour ContractAccount
      const { admin } = schemas.contractAccountSchema.decode(accountInfo.data);
      console.log("ContractAccount admin", { admin });
      break;
    default:
      console.log(`Type de compte inconnu: ${account_type}`);
  }

    });
   
}

async function main() {
    
   await fetchContractState(pg.connection);

   
}

async function initializeKeypair(connection: Web3.Connection): Promise<Web3.Keypair> {
    if (!process.env.PRIVATE_KEY) {
        console.log('Generating new keypair... 🗝️');
        const signer = Web3.Keypair.generate();
        console.log('Creating .env file');
        fs.writeFileSync('.env', `PRIVATE_KEY=[${signer.secretKey.toString()}]`);
        await airdropSolIfNeeded(signer, connection);
        return signer;
    }

    const secret = JSON.parse(process.env.PRIVATE_KEY ?? '') as number[];
    const secretKey = Uint8Array.from(secret);
    const keypairFromSecret = Web3.Keypair.fromSecretKey(secretKey);
    await airdropSolIfNeeded(keypairFromSecret, connection);
    return keypairFromSecret;

}

async function airdropSolIfNeeded(
    signer: Web3.Keypair,
    connection: Web3.Connection
) {
    const balance = await connection.getBalance(signer.publicKey);
    console.log('Current balance is', balance / Web3.LAMPORTS_PER_SOL, 'SOL');

    // 1 SOL should be enough for almost anything
    if (balance / Web3.LAMPORTS_PER_SOL < 1) {
        // Can only get up to 2 SOL per request 
        console.log('Airdropping 1 SOL');
        const airdropSignature = await connection.requestAirdrop(
            signer.publicKey,
            Web3.LAMPORTS_PER_SOL
        );

        const latestBlockhash = await connection.getLatestBlockhash();

        await connection.confirmTransaction({
            blockhash: latestBlockhash.blockhash,
            lastValidBlockHeight: latestBlockhash.lastValidBlockHeight,
            signature: airdropSignature,
        });

        const newBalance = await connection.getBalance(signer.publicKey);
        console.log('New balance is', newBalance / Web3.LAMPORTS_PER_SOL, 'SOL');
    }
}

// async function pingProgram(connection: Web3.Connection, payer: Web3.Keypair) {
//     const transaction = new Web3.Transaction()
//     const instruction = new Web3.TransactionInstruction({
//         keys: [
//             {
//                 pubkey: PROGRAM_DATA_PUBLIC_KEY,
//                 isSigner: false,
//                 isWritable: true
//             }
//         ],

//     })

//     transaction.add(instruction)

//     const transactionSignature = await Web3.sendAndConfirmTransaction(connection, transaction, [payer])

//     console.log(
//         `Transaction https://explorer.solana.com/tx/${transactionSignature}?cluster=devnet`
//     )
// }

main()
    .then(() => {
        console.log('Finished successfully');
        process.exit(0);
    })
    .catch((error) => {
        console.log(error);
        process.exit(1);
    });
