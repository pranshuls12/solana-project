import * as anchor from '@project-serum/anchor';
import { PublicKey, SystemProgram, LAMPORTS_PER_SOL, Keypair, sendAndConfirmTransaction,Transaction,ComputeBudgetProgram } from '@solana/web3.js';
import { Program  } from '@project-serum/anchor';
import { NeptuneLbp } from "../target/types/neptune_lbp";
import { fetchContractState } from '../app/fetchContractState';
import * as fs from 'fs';
import * as path from 'path';
import * as  solanaWeb3 from '@solana/web3.js' ;
import { TOKEN_PROGRAM_ID, createMint, ASSOCIATED_TOKEN_PROGRAM_ID, getAssociatedTokenAddress,getAssociatedTokenAddressSync, getOrCreateAssociatedTokenAccount, mintTo } from '@solana/spl-token';
import TransactionFactory from '@project-serum/anchor/dist/cjs/program/namespace/transaction';

// Function to load a keypair from a JSON file
function loadKeypairFromFile(filePath) {
  // Read the file content
  const fileContent = fs.readFileSync(filePath, { encoding: 'utf8' });

  // Parse the JSON content to an array
  const secretKey = JSON.parse(fileContent);

  // Create a keypair from the secret key
  const keypair = Keypair.fromSecretKey(new Uint8Array(secretKey));
  return keypair;
}
async function createAtaForUser(mint, userKeypair, provider) {
  const res = await getOrCreateAssociatedTokenAccount(
    provider.connection,
    provider.wallet.payer,// The payer of the transaction; also the owner of the ATA 
    mint, // The mint address of the token for which you're creating the ATA
    userKeypair.publicKey, // The owner of the newly created ATA
    // Pass additional options if necessary
  );

  return res
}
async function requestAirdrop(connection, publicKey, lamports) {
  const airdropSignature = await connection.requestAirdrop(
    publicKey,
    lamports,
  );

  await connection.confirmTransaction(airdropSignature);
}
function sleep(ms) {
  return new Promise(resolve => setTimeout(resolve, ms));
}

describe("neptune-lbp", () => {

  // Specify the path to the JSON file
  const keypairPath = path.join(__dirname, '../wallets/NeptkHfKK36uWkxTSGuta1gzwnPXjZSkmCsD7NW54ib.json');

  // Load the keypair
  const myKeypair = loadKeypairFromFile(keypairPath);
  // Configure the connection to the local Solana node
  const localnetConnection = new anchor.web3.Connection(
    // This URL assumes you're running a local Solana node on the default port
    "http://localhost:8899",
    "confirmed" // Use "confirmed" state for the commitment level
  );

  // creating copies to simplify further test creation
  // Pool Owner Sells InputToken ,  Buys OutputToken
  let _inputTokenMint
  let _outputTokenMint
  let _poolAccountPda
  let _ownerInputAta
  let _ownerOutputAta
  let _ownerBpAta
  let _aliceInputAta
  let _aliceOutputAta
  let _aliceBpAta
  let _bobInputAta
  let _bobOutputAta
  let _poolInputAta
  let _poolOutputAta
  let _poolBpAta
  let _bpTokenMint
  let _masterAccountPda
  let _masterInputAta
  let _masterOutputAta
  let _adminInputAta
  let _adminOutputAta

  const aliceKeyPath = path.join(__dirname, '../wallets/ACQXYUiRwziuoeXNeJyZ8bB2fdkmU7ftaiDLbmEqkSYW.json');
  const bobKeyPath = path.join(__dirname, '../wallets/NeptkHfKK36uWkxTSGuta1gzwnPXjZSkmCsD7NW54ib.json');
  const aliceKeyPair = loadKeypairFromFile(aliceKeyPath)
  const bobKeyPair = loadKeypairFromFile(bobKeyPath)
  // creates first instruction
  const modifyComputeUnits = ComputeBudgetProgram.setComputeUnitLimit({ 
    units: 1400000 
  });
  const addPriorityFee = ComputeBudgetProgram.setComputeUnitPrice({ 
    microLamports: 1 
  });


  const wallet = new anchor.Wallet(myKeypair);

  // Configure the provider with the connection and wallet
  const provider = new anchor.AnchorProvider(localnetConnection, wallet, {
    skipPreflight: true,
  });

  // Set the provider for your Anchor program
  anchor.setProvider(provider);

  const idl = JSON.parse(fs.readFileSync('./target/idl/neptune_lbp.json', 'utf8'));
  const programId = new PublicKey("3Wxsikr3N9wJAiKcHfHD5ALyYEogkiGTi2u6nUFm5x3F");

  const program = new Program(idl, programId, provider);
  const [masterAccountPda, _] = PublicKey.findProgramAddressSync(
    [Buffer.from("master_account")],
    programId
  );
  it("Master Account is initialized!", async () => {
    const tx = await program.methods.initialize(provider.wallet.publicKey ).accounts({
      masterAccount: masterAccountPda,
      admin: provider.wallet.publicKey,
      systemProgram: SystemProgram.programId,
    }).rpc();
    console.log("Your transaction signature", tx);
  });
  it("Admin can set fees!", async () => {
    let swap_fee = new anchor.BN(2);
    let flat_rate= new anchor.BN(3);
    const tx = await program.methods.setFeePercentage(swap_fee,flat_rate).accounts({
      masterAccount: masterAccountPda,
      admin: provider.wallet.publicKey,
      systemProgram: SystemProgram.programId,
    }).rpc();
    console.log("Your transaction signature", tx);
  });
  it("Admin can revoke his privileges for somebody else!", async () => {
    let admin = provider.wallet.publicKey;
    const tx = await program.methods.setAdmin(admin).accounts({
      masterAccount: masterAccountPda,
      admin: provider.wallet.publicKey,
      systemProgram: SystemProgram.programId,
    }).rpc();
    console.log("Your transaction signature", tx);
  });
  it("Admin can change the address that can extract the fees from treasury!", async () => {
    let admin = provider.wallet.publicKey;
    const tx = await program.methods.setFeeCollector(admin).accounts({
      masterAccount: masterAccountPda,
      admin: provider.wallet.publicKey,
      systemProgram: SystemProgram.programId,
    }).rpc();
    console.log("Your transaction signature", tx);
  });
  it("Verify invariant calculation", async () => {

    const normalizedWeights = Buffer.from(new Uint8Array([90, 10]));
    //Selling 1 000 000 000 tokens and placing a collateral of 10 000
    const balances = [new anchor.BN(100000000000), new anchor.BN(10000000000)];

    // creates first instruction
    const modifyComputeUnits = ComputeBudgetProgram.setComputeUnitLimit({ 
      units: 1400000 
    });
    const addPriorityFee = ComputeBudgetProgram.setComputeUnitPrice({ 
      microLamports: 1 
    });


    // creates second instruction
    const instructionTwo = await program.methods
    .calculateInvariantInstruction(normalizedWeights, balances)
    .accounts({})
    .instruction()

    // add both instruction to one transaction
    const transaction = new Transaction().add(modifyComputeUnits)
    .add(addPriorityFee)
    .add(instructionTwo)

// send transaction
    const tx = await sendAndConfirmTransaction(provider.connection,transaction,[wallet.payer],{
      skipPreflight: true,
    } )
   // const tx = await program.methods.calculateInvariantInstruction(normalizedWeights, balances)
     // .rpc();
    console.log("Transaction signature:", tx);
  });
  it('Initializes the pool funds', async () => {
    // Use dynamically generated mint addresses for input and output tokens
    const inputTokenMint = await createMint(
      provider.connection,
      wallet.payer, // Payer of the transaction
      provider.wallet.publicKey, // Authority of the mint
      null, // Freeze authority (null if not used)
      9, // Decimals

    );
    _inputTokenMint = inputTokenMint
    // Create the output token mint
    const outputTokenMint = await createMint(
      provider.connection,
      wallet.payer, // Payer of the transaction
      provider.wallet.publicKey, // Authority of the mint
      null, // Freeze authority (null if not used)
      9, // Decimals

    );
    _outputTokenMint = outputTokenMint
    // Derive the address and bump seed for the Pool account PDA
    const [poolAccountPda, poolAccountBump] = await anchor.web3.PublicKey.findProgramAddress(
      [Buffer.from('pool_account'), provider.wallet.publicKey.toBuffer(), inputTokenMint.toBuffer()],
      program.programId
    );
    _poolAccountPda = poolAccountPda
    // Derive the address and bump seed for the Pool account PDA
    const [masterAccountPda, masterAccountBump] = await anchor.web3.PublicKey.findProgramAddress(
      [Buffer.from('master_account')],
      program.programId
    );
    _masterAccountPda = masterAccountPda;
    // Derive ATAs for input and output tokens for the pool account
    const inputTokenAta = await getAssociatedTokenAddress(
      inputTokenMint,
      poolAccountPda, // Imported from '@solana/spl-token'
      true,
      TOKEN_PROGRAM_ID, // Imported from '@solana/spl-token'
      ASSOCIATED_TOKEN_PROGRAM_ID,

    );
    _poolInputAta = inputTokenAta

    const outputTokenAta = await getAssociatedTokenAddressSync(
      outputTokenMint,
      poolAccountPda, // Imported from '@solana/spl-token'
      true,
      TOKEN_PROGRAM_ID,
      ASSOCIATED_TOKEN_PROGRAM_ID,
    );
    _poolOutputAta = outputTokenAta
    // Derive the address and bump seed for the BP token mint PDA
    const [bpTokenMint, bpTokenMintBump] = await anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from('bp_token_mint'), poolAccountPda.toBuffer()],
      program.programId
    );
    _bpTokenMint = bpTokenMint

    // Parameters for initializing the pool
    const params = {
      accountType: 0, // Example value
      startTimestamp: new anchor.BN(Date.now() / 1000+1), // Current timestamp
      endTimestamp: new anchor.BN(Date.now() / 1000 + 11), // One hour later
      startWeights: [new anchor.BN(90), new anchor.BN(10)], // Example weights
      endWeights: [new anchor.BN(10), new anchor.BN(90)], // Example weights
      isSol: false
    };
    if (params.isSol) {

    // Convert SOL amount to lamports
    const amountLamports = solanaWeb3.LAMPORTS_PER_SOL * 1;

    // Create the transaction instruction for transferring SOL
    const transferInstruction = solanaWeb3.SystemProgram.transfer({
        fromPubkey: provider.wallet.publicKey,
        toPubkey: poolAccountPda,
        lamports: amountLamports,
    });

    // Create a transaction object and add the transfer instruction
    const transaction = new solanaWeb3.Transaction().add(transferInstruction);

    // Send and confirm the transaction
    const signature = await solanaWeb3.sendAndConfirmTransaction(  provider.connection, transaction, [myKeypair]);
    console.log('Sol amount in pool initialised with signature:', signature);

    }


    const masterInputTokenAta = await getOrCreateAssociatedTokenAccount(
      provider.connection,
      myKeypair,
      inputTokenMint,
      masterAccountPda, // Imported from '@solana/spl-token'
      true, undefined, undefined,
      TOKEN_PROGRAM_ID,
      ASSOCIATED_TOKEN_PROGRAM_ID,
    );
    const masterOutputTokenAta = await getOrCreateAssociatedTokenAccount(
      provider.connection,
      myKeypair,
      outputTokenMint,
      masterAccountPda, // Imported from '@solana/spl-token'
      true, undefined, undefined,
      TOKEN_PROGRAM_ID,
      ASSOCIATED_TOKEN_PROGRAM_ID,
    );
    _masterInputAta=masterInputTokenAta
    _masterOutputAta=masterOutputTokenAta
    // Transaction to initialize the pool
    let instructionTwo =  await program.methods.initializePool(params)
      .accounts({
        user: provider.wallet.publicKey,
        inputTokenMint: inputTokenMint,
        outputTokenMint: outputTokenMint,
        masterAccount: masterAccountPda,
        masterAccountInputFeeAta:masterInputTokenAta.address,
        masterAccountOutputFeeAta:masterOutputTokenAta.address,
        poolAccount: poolAccountPda, // Use the derived PDA
       bpTokenMint: bpTokenMint, // Assume this is correctly derived elsewhere
        associatedTokenProgram: anchor.utils.token.ASSOCIATED_PROGRAM_ID,
        tokenProgram: anchor.utils.token.TOKEN_PROGRAM_ID,
        systemProgram: SystemProgram.programId,
      }).instruction()


    // add both instruction to one transaction
    let transaction = new Transaction().add(modifyComputeUnits)
    .add(addPriorityFee)
    .add(instructionTwo)

    // send transaction
    let tx = await sendAndConfirmTransaction(provider.connection,transaction,[wallet.payer],{
      skipPreflight: true,
    } )
   // const tx = await program.methods.calculateInvariantInstruction(normalizedWeights, balances)
     // .rpc();
    console.log("Transaction signature:", tx);
    console.log("Your transaction signature for initializing the pool", tx);

    // Create ATAs for user's input and output tokens
    const userInputTokenAta = await getOrCreateAssociatedTokenAccount(
      provider.connection,
      wallet.payer,
      inputTokenMint,
      provider.wallet.publicKey,
    );
    _ownerInputAta = userInputTokenAta
    const userOutputTokenAta = await getOrCreateAssociatedTokenAccount(
      provider.connection,
      wallet.payer,
      outputTokenMint,
      provider.wallet.publicKey,
    );
    _ownerOutputAta = userOutputTokenAta

    // Mint some tokens to user's ATAs to simulate having funds to initialize the pool
    await mintTo(
      provider.connection,
      wallet.payer, // payer
      inputTokenMint, // mint address
      userInputTokenAta.address, // destination
      provider.wallet.publicKey, // mint authority
      1000000000000, // amount: 10000 tokens considering 9 decimals
    );

    await mintTo(
      provider.connection,
      wallet.payer, // payer
      outputTokenMint, // mint address
      userOutputTokenAta.address, // destination
      provider.wallet.publicKey, // mint authority
      1000000000000, // amount: 1000 tokens considering 9 decimals
    );

    // Parameters for initializing the pool funds
    const initPoolFundsParams = {
      balances: [new anchor.BN(900000000000), new anchor.BN(100000000000)], // Example initial amounts for input and output tokens
      normalizedWeights: [new anchor.BN(90), new anchor.BN(10)], // Example normalized weights
    };

    const bpTokenAta = await getOrCreateAssociatedTokenAccount(
      provider.connection,
      myKeypair,
      bpTokenMint,
      poolAccountPda, // Imported from '@solana/spl-token'
      true, undefined, undefined,
      TOKEN_PROGRAM_ID,
      ASSOCIATED_TOKEN_PROGRAM_ID,
    );
    const ownerBpAta = await getOrCreateAssociatedTokenAccount(
      provider.connection,
      myKeypair,
      bpTokenMint,
      myKeypair.publicKey, // Imported from '@solana/spl-token'
      false, undefined, undefined,
      TOKEN_PROGRAM_ID,
      ASSOCIATED_TOKEN_PROGRAM_ID,
    );
    _poolBpAta = bpTokenAta
    _ownerBpAta = ownerBpAta
    // Transaction to initialize the pool funds

 
    instructionTwo = await program.methods.initializePoolFunds(initPoolFundsParams)
      .accounts({
        user: provider.wallet.publicKey,
        inputTokenMint: inputTokenMint,
        outputTokenMint: outputTokenMint,
        bpTokenMint: bpTokenMint,
        poolAccount: poolAccountPda,
        userInputAta: userInputTokenAta.address,
        userOutputAta: userOutputTokenAta.address,
        userBpAta: ownerBpAta.address,
        poolInputAta: inputTokenAta, // Assuming these are already derived or created
        poolOutputAta: outputTokenAta,
        tokenProgram: anchor.utils.token.TOKEN_PROGRAM_ID,
        systemProgram: SystemProgram.programId,
      }).instruction()
       // add both instruction to one transaction
    transaction = new Transaction().add(modifyComputeUnits)
    .add(addPriorityFee)
    .add(instructionTwo)

    // send transaction
    const tx2 = await sendAndConfirmTransaction(provider.connection,transaction,[myKeypair],{
      skipPreflight: true,
    } )
    const poolInputTokenBalanceAfter = await provider.connection.getTokenAccountBalance(_poolInputAta);
    const poolOutputTokenBalanceAfter = await provider.connection.getTokenAccountBalance(_poolOutputAta);

    const ownerBpTokenBalanceAfter = await provider.connection.getTokenAccountBalance(ownerBpAta.address);

    
    console.log("Transaction signature for initializing the pool funds", tx);
    console.log(`Pool Balance after initialisation:`)
    console.log(`Input Token ${poolInputTokenBalanceAfter.value.amount}`);
    console.log(`Output Token ${poolOutputTokenBalanceAfter.value.amount}`);
    console.log(`User Bp Token ${ownerBpTokenBalanceAfter.value.amount}`);

  
  });
// Allow the owner to  add liquidity
  it('Allows the owner to add liquidity', async () => {

    const ownerBpTokenBalanceBefore= await provider.connection.getTokenAccountBalance(_ownerBpAta.address);
    const userInputTokenBalanceBefore = await provider.connection.getTokenAccountBalance(_ownerInputAta.address);
    const userOutputTokenBalanceBefore = await provider.connection.getTokenAccountBalance(_ownerOutputAta.address);
  
    const amountOuputToken = new anchor.BN(10000000000); // 10 tokens, assuming 9 decimals
   
    
    // Call join_pool
    const instruction = await program.methods.joinPool(amountOuputToken)
      .accounts({
        user:wallet.publicKey,
        outputTokenMint:_outputTokenMint,
        userOutputAta: _ownerOutputAta.address, // The user's input token ATA
        userBpAta: _ownerBpAta.address, // The user's BP token ATA, needs to be created if it doesn't exist
        inputTokenMint: _inputTokenMint,
        poolAccount: _poolAccountPda, // The pool account PDA
        poolInputAta: _poolInputAta, // The pool's input token ATA
        poolOutputAta: _poolOutputAta, // The pool's input token ATA
        poolBpAta: _poolBpAta.address, // The pool's BP token ATA
        bpTokenMint: _bpTokenMint, // The BP token mint
        tokenProgram: TOKEN_PROGRAM_ID,
      }).signers([myKeypair]).instruction();

    let transaction = new Transaction().add(modifyComputeUnits)
    .add(addPriorityFee)
    
    .add(instruction)

    // send transaction
    const txJoin = await sendAndConfirmTransaction(provider.connection,transaction,[wallet.payer],{
      skipPreflight: true,
    } )
    console.log("Transaction signature for joining the pool", txJoin);

    // Optional: Verify the user's new balances
    // Fetch the user's input token and BP token balances after joining the pool
    const userInputTokenBalanceAfter = await provider.connection.getTokenAccountBalance(_ownerInputAta.address);
    const userOutputTokenBalanceAfter = await provider.connection.getTokenAccountBalance(_ownerOutputAta.address);
    const ownerBpTokenBalanceAfter= await provider.connection.getTokenAccountBalance(_ownerBpAta.address);
   
    console.log(`Owner's input token balance before joining the pool: ${userInputTokenBalanceBefore.value.amount}`);
    console.log(`Owners's input token balance after joining the pool: ${userInputTokenBalanceAfter.value.amount}`);
    console.log(`Owner's ouput token balance before joining the pool: ${userOutputTokenBalanceBefore.value.amount}`);
    console.log(`Owners's ouput token balance after joining the pool: ${userOutputTokenBalanceAfter.value.amount}`); 
    console.log(`Owner's bp token balance before joining the pool: ${ownerBpTokenBalanceBefore.value.amount}`);
    console.log(`Owners's bp token balance after joining the pool: ${ownerBpTokenBalanceAfter.value.amount}`);
   
    // Add assertions to verify that the balances have updated as expected
    // For example, using chai:
    // expect(new anchor.BN(userInputTokenBalanceAfter.value.amount)).to.be.lessThan(new anchor.BN(1000000000)); // User should have less input tokens now
    // expect(new anchor.BN(userBpTokenBalanceAfter.value.amount)).to.be.greaterThan(new anchor.BN(0)); // User should have received BP tokens

  });
  
  // Allow a user to  swap a token
  it('Allows a user to Swap', async () => {
    await sleep(3000);
    const aliceOutputTokenAta = await createAtaForUser(_outputTokenMint, aliceKeyPair, provider);
    const aliceBpTokenAta = await createAtaForUser(_bpTokenMint, aliceKeyPair, provider);

    const bobOutputTokenAta = await createAtaForUser(_outputTokenMint, bobKeyPair, provider);
    const bobBpTokenAta = await createAtaForUser(_bpTokenMint, bobKeyPair, provider);
    _aliceOutputAta = aliceOutputTokenAta
    _aliceBpAta = aliceBpTokenAta
    _bobOutputAta = bobOutputTokenAta
    // Request an airdrop for Alice and Bob
    console.log("Requesting airdrop");
    await requestAirdrop(provider.connection, aliceKeyPair.publicKey, 10e9); // Request 1 SOL (1e9 lamports)
    await requestAirdrop(provider.connection, bobKeyPair.publicKey, 10e9); // Request 1 SOL (1e9 lamports)
    // Ensure the user has enough output tokens 
    // Mint some tokens to user's ATAs to simulate having funds to initialize the pool
    await mintTo(
      provider.connection,
      wallet.payer, // payer
      _outputTokenMint, // mint address
      aliceOutputTokenAta.address, // destination
      provider.wallet.publicKey, // mint authority
      1000000000000, // amount: 10000 tokens considering 9 decimals
    );

    await mintTo(
      provider.connection,
      wallet.payer, // payer
      _outputTokenMint, // mint address
      bobOutputTokenAta.address, // destination
      provider.wallet.publicKey, // mint authority
      1000000000000, // amount: 10000 tokens considering 9 decimals
    );
    const aliceInputTokenAta = await createAtaForUser(_inputTokenMint, aliceKeyPair, provider);
    _aliceInputAta=aliceInputTokenAta
    const userInputTokenBalanceBefore = await provider.connection.getTokenAccountBalance(_aliceInputAta.address);
    const userOutputTokenBalanceBefore = await provider.connection.getTokenAccountBalance(_aliceOutputAta.address);
  
    const amountOuputToken = new anchor.BN(10000000000); // 500 tokens, assuming 9 decimals
   /* console.log({
      owner: provider.wallet.publicKey,
      user: aliceKeyPair.publicKey,
      userOutputAta: _aliceOutputAta, // The user's input token ATA
      userInputtAta: aliceInputTokenAta, // The user's input token ATA
      userBpAta: _aliceBpAta.address, // The user's BP token ATA, needs to be created if it doesn't exist
      inputTokenMint: _inputTokenMint,
      poolAccount: _poolAccountPda, // The pool account PDA
      poolInputAta: _poolInputAta, // The pool's input token ATA
      poolOutputAta: _poolOutputAta, // The pool's input token ATA
      poolBpAta: _poolBpAta.address, // The pool's BP token ATA
      bpTokenMint: _bpTokenMint, // The BP token mint
      tokenProgram: TOKEN_PROGRAM_ID,
    })*/
    // Call join_pool
   
  const instruction = await program.methods.buySwap(amountOuputToken,true)
      .accounts({
        owner: provider.wallet.publicKey,
        outputTokenMint:_outputTokenMint,
        masterAccount: _masterAccountPda,
        user: aliceKeyPair.publicKey,
        userOutputAta: _aliceOutputAta.address, // The user's input token ATA
        userInputAta: aliceInputTokenAta.address, // The user's input token ATA
        userBpAta: _aliceBpAta.address, // The user's BP token ATA, needs to be created if it doesn't exist
        inputTokenMint: _inputTokenMint,
        poolAccount: _poolAccountPda, // The pool account PDA
        poolInputAta: _poolInputAta, // The pool's input token ATA
        poolOutputAta: _poolOutputAta, // The pool's input token ATA
        poolBpAta: _poolBpAta.address, // The pool's BP token ATA
        bpTokenMint: _bpTokenMint, // The BP token mint,
        feeCollectorInputAta: _masterInputAta.address, 
        associatedTokenProgram: anchor.utils.token.ASSOCIATED_PROGRAM_ID,
        tokenProgram: anchor.utils.token.TOKEN_PROGRAM_ID,
        systemProgram: SystemProgram.programId,
 
      })
      .signers([aliceKeyPair])
      .instruction();

    let transaction = new Transaction().add(modifyComputeUnits)
    .add(addPriorityFee)
    .add(instruction)

    // send transaction
    const txJoin = await sendAndConfirmTransaction(provider.connection,transaction,[aliceKeyPair],{
      skipPreflight: true,
    } )
    console.log("Transaction signature for  swapping into the pool", txJoin);

     const userInputTokenBalanceAfter = await provider.connection.getTokenAccountBalance(_aliceInputAta.address);
    const userOutputTokenBalanceAfter = await provider.connection.getTokenAccountBalance(_aliceOutputAta.address);
   
    console.log(`Swapper's input token balance before joining the pool: ${userInputTokenBalanceBefore.value.amount}`);
    console.log(`Swapper's input token balance after joining the pool: ${userInputTokenBalanceAfter.value.amount}`);
    console.log(`Swapper's ouput token balance before joining the pool: ${userOutputTokenBalanceBefore.value.amount}`);
    console.log(`Swapper's ouput token balance after joining the pool: ${userOutputTokenBalanceAfter.value.amount}`); 
    
  });

  it('Allows the owner to redeem his token ', async () => {
    await sleep(3000);
    const ownerBpTokenBalanceBefore= await provider.connection.getTokenAccountBalance(_ownerBpAta.address);
    const poolInputTokenBalanceAfter = await provider.connection.getTokenAccountBalance(_poolInputAta);
    const poolOutputTokenBalanceAfter = await provider.connection.getTokenAccountBalance(_poolOutputAta);
   
    console.log(`Owner Bp Token Before${ownerBpTokenBalanceBefore.value.amount}`);
    console.log(`Pool balance before : in ${poolInputTokenBalanceAfter.value.amount} out ${poolOutputTokenBalanceAfter.value.amount}`);

    const amountBpToken = new anchor.BN(ownerBpTokenBalanceBefore.value.amount); // 500 tokens, assuming 9 decimals
   /* console.log({
      owner: provider.wallet.publicKey,
      user: aliceKeyPair.publicKey,
      userOutputAta: _aliceOutputAta, // The user's input token ATA
      userInputtAta: aliceInputTokenAta, // The user's input token ATA
      userBpAta: _aliceBpAta.address, // The user's BP token ATA, needs to be created if it doesn't exist
      inputTokenMint: _inputTokenMint,
      poolAccount: _poolAccountPda, // The pool account PDA
      poolInputAta: _poolInputAta, // The pool's input token ATA
      poolOutputAta: _poolOutputAta, // The pool's input token ATA
      poolBpAta: _poolBpAta.address, // The pool's BP token ATA
      bpTokenMint: _bpTokenMint, // The BP token mint
      tokenProgram: TOKEN_PROGRAM_ID,
    })*/
    // Call join_pool
   
   const instruction = await program.methods.redeemBpTokens(amountBpToken)
      .accounts({
        owner: provider.wallet.publicKey,
        masterAccount: _masterAccountPda,
        user: provider.wallet.publicKey,
        userInputAta: _ownerInputAta.address, // The user's input token ATA
        userOutputAta: _ownerOutputAta.address, // The user's input token ATA
        userBpAta: _ownerBpAta.address, // The user's BP token ATA, needs to be created if it doesn't exist
        inputTokenMint: _inputTokenMint,
        outputTokenMint: _outputTokenMint,
        poolAccount: _poolAccountPda, // The pool account PDA
        poolInputAta: _poolInputAta, // The pool's input token ATA
        poolOutputAta: _poolOutputAta, // The pool's input token ATA
        poolBpAta: _poolBpAta.address, // The pool's BP token ATA
        bpTokenMint: _bpTokenMint, // The BP token mint
        feeCollectorOutputAta: _masterOutputAta.address, 
        associatedTokenProgram: anchor.utils.token.ASSOCIATED_PROGRAM_ID,
        tokenProgram: anchor.utils.token.TOKEN_PROGRAM_ID,
        systemProgram: SystemProgram.programId,
      })
      .signers([myKeypair])
      .instruction();

    let transaction = new Transaction().add(modifyComputeUnits)
    .add(addPriorityFee)
    .add(instruction)

    // send transaction
    const txJoin = await sendAndConfirmTransaction(provider.connection,transaction,[myKeypair],{
      skipPreflight: true,
    } )
    console.log("Transaction signature for  swapping into the pool", txJoin);

    // Optional: Verify the user's new balances
    // Fetch the user's input token and BP token balances after joining the pool
    const userInputTokenBalanceAfter = await provider.connection.getTokenAccountBalance(_ownerInputAta.address);
    const userOutputTokenBalanceAfter = await provider.connection.getTokenAccountBalance(_ownerOutputAta.address);
    const userBpTokenBalanceAfter = await provider.connection.getTokenAccountBalance( _ownerBpAta.address);

    console.log(`User's input token balance after joining the pool: ${userInputTokenBalanceAfter.value.amount}`);
    console.log(`User's output token balance after joining the pool: ${userOutputTokenBalanceAfter.value.amount}`);
    
    console.log(`User's BP token balance after joining the pool: ${userBpTokenBalanceAfter.value.amount}`);

  });
  it("feeCollector Can collect inToken fees!", async () => {
    let admin = provider.wallet.publicKey;
    const masterInputTokenBalanceBefore = await provider.connection.getTokenAccountBalance(_masterInputAta.address);
    const collectorInputTokenBalanceBefore = await provider.connection.getTokenAccountBalance(_ownerInputAta.address);
    
    const tx = await program.methods.collectFeesFromAta().accounts({
      masterAccount: masterAccountPda,
      feeCollectorTokenAta:_ownerInputAta.address,
      masterAccountTokenAta:_masterInputAta.address,
      tokenMint:_inputTokenMint,
      associatedTokenProgram: anchor.utils.token.ASSOCIATED_PROGRAM_ID,
      tokenProgram:  anchor.utils.token.TOKEN_PROGRAM_ID,
      systemProgram: SystemProgram.programId,
    }).rpc();
    console.log("Your transaction signature", tx);
    const masterInputTokenBalanceAfter = await provider.connection.getTokenAccountBalance(_masterInputAta.address);
    const collectorInputTokenBalanceAfter = await provider.connection.getTokenAccountBalance(_ownerInputAta.address);
    
    console.log(``)
    console.log(`Collected Input Token Fees by master account ${masterInputTokenBalanceBefore.value.amount}`);
    console.log(`Master fee account Before${masterInputTokenBalanceBefore.value.amount}`);
    console.log(`Fee collector balance before ${collectorInputTokenBalanceBefore.value.amount}`);
    console.log(`Fee collector balance after ${collectorInputTokenBalanceAfter.value.amount}`);
    
  });
  it("feeCollector Can collect outToken fees!", async () => {
    let admin = provider.wallet.publicKey;
    const masterInputTokenBalanceBefore = await provider.connection.getTokenAccountBalance(_masterOutputAta.address);
    const collectorInputTokenBalanceBefore = await provider.connection.getTokenAccountBalance(_ownerOutputAta.address);
    
    const tx = await program.methods.collectFeesFromAta().accounts({
      masterAccount: masterAccountPda,
      feeCollectorTokenAta:_ownerOutputAta.address,
      masterAccountTokenAta:_masterOutputAta.address,
      tokenMint:_outputTokenMint,
      associatedTokenProgram: anchor.utils.token.ASSOCIATED_PROGRAM_ID,
      tokenProgram:  anchor.utils.token.TOKEN_PROGRAM_ID,
      systemProgram: SystemProgram.programId,
    }).rpc();
    console.log("Your transaction signature", tx);
    const masterInputTokenBalanceAfter = await provider.connection.getTokenAccountBalance(_masterOutputAta.address);
    const collectorInputTokenBalanceAfter = await provider.connection.getTokenAccountBalance(_ownerOutputAta.address);
    
    console.log(``)
    console.log(`Collected Output Token Fees by master account ${masterInputTokenBalanceBefore.value.amount}`);
    console.log(`Master fee account Before${masterInputTokenBalanceBefore.value.amount}`);
    console.log(`Fee collector balance before ${collectorInputTokenBalanceBefore.value.amount}`);
    console.log(`Fee collector balance after ${collectorInputTokenBalanceAfter.value.amount}`);
    
  });

  /*it('Allows a user to redeem the token', async () => {
    const aliceInputTokenAta = await createAtaForUser(_inputTokenMint, aliceKeyPair, provider);
    const bobInputTokenAta = await createAtaForUser(_inputTokenMint, bobKeyPair, provider);

    await requestAirdrop(provider.connection, aliceKeyPair.publicKey, 10e9); // Request 1 SOL (1e9 lamports)
    await requestAirdrop(provider.connection, bobKeyPair.publicKey, 10e9); // Request 1 SOL (1e9 lamports)


    // Specify the amount of input tokens the user will contribute to join the pool
    const amountBpToken = new anchor.BN(400000000); // 500 tokens, assuming 9 decimals
    console.log(_poolInputAta)
    const userBpTokenBalanceBefore = await provider.connection.getTokenAccountBalance(_aliceBpAta.address);
    const poolInputTokenBalanceBefore = await provider.connection.getTokenAccountBalance(_poolInputAta);

    console.log(`User's BP token balance before joining the pool: ${userBpTokenBalanceBefore.value.amount}`);
    console.log(`Pool's Input token balance before joining the pool: ${poolInputTokenBalanceBefore.value.amount}`);


    // Call join_pool
    const txJoin = await program.methods.redeemBpTokens(amountBpToken)
      .accounts({
        owner: provider.wallet.publicKey,
        user: aliceKeyPair.publicKey,
        userInputAta: aliceInputTokenAta.address, // The user's input token ATA
        userBpAta: _aliceBpAta.address, // The user's BP token ATA, needs to be created if it doesn't exist
        inputTokenMint: _inputTokenMint, // used to derive pool account
        poolAccount: _poolAccountPda, // The pool account PDA
        poolInputAta: _poolInputAta, // The pool's input token ATA
        poolBpAta: _poolBpAta.address, // The pool's BP token ATA
        bpTokenMint: _bpTokenMint, // The BP token mint
        tokenProgram: TOKEN_PROGRAM_ID,
      })
      .signers([aliceKeyPair])
      .rpc();
    console.log("Transaction signature for joining the pool", txJoin);

    // Optional: Verify the user's new balances
    // Fetch the user's input token and BP token balances after joining the pool
    const userInputTokenBalanceAfter = await provider.connection.getTokenAccountBalance(aliceInputTokenAta.address);
    const userBpTokenBalanceAfter = await provider.connection.getTokenAccountBalance(_aliceBpAta.address);

    console.log(`User's input token balance after joining the pool: ${userInputTokenBalanceAfter.value.amount}`);
    console.log(`User's BP token balance after joining the pool: ${userBpTokenBalanceAfter.value.amount}`);

    // Add assertions to verify that the balances have updated as expected
    // For example, using chai:
    // expect(new anchor.BN(userInputTokenBalanceAfter.value.amount)).to.be.lessThan(new anchor.BN(1000000000)); // User should have less input tokens now
    // expect(new anchor.BN(userBpTokenBalanceAfter.value.amount)).to.be.greaterThan(new anchor.BN(0)); // User should have received BP tokens

  });
/*
  it('Allows the owner to exit the pool', async () => {
    
    // get the pool  final balances 
    const poolInputTokenBalanceBefore = await provider.connection.getTokenAccountBalance(_poolInputAta);
    const poolOuputTokenBalanceBefore = await provider.connection.getTokenAccountBalance(_poolOutputAta);


    // Call join_pool
    const txJoin = await program.methods.ownerExit(new anchor.BN(poolInputTokenBalanceBefore.value.amount),new anchor.BN(poolOuputTokenBalanceBefore.value.amount))
      .accounts({
        owner: provider.wallet.publicKey,
        userInputAta: _ownerInputAta.address, // The user's input token ATA
        userOutputAta: _ownerOutputAta.address, // The user's input token ATA
        inputTokenMint: _inputTokenMint, // used to derive pool account
        poolAccount: _poolAccountPda, // The pool account PDA
        poolInputAta: _poolInputAta, // The pool's input token ATA
        poolOutputAta: _poolOutputAta , // The pool's BP token ATA
        tokenProgram: TOKEN_PROGRAM_ID,
      })
      .rpc();
    console.log("Transaction signature for joining the pool", txJoin);

     // get the pool  final balances 
     const poolInputTokenBalanceAfter = await provider.connection.getTokenAccountBalance(_poolInputAta);
     const poolOuputTokenBalanceAfter = await provider.connection.getTokenAccountBalance(_poolOutputAta);
 
    console.log(`Pool input balance after owner exit the pool: ${poolInputTokenBalanceAfter.value.amount}`);
    console.log(`Pool output token after owner exit the pool: ${poolOuputTokenBalanceAfter.value.amount}`);

    // Add assertions to verify that the balances have updated as expected
    // For example, using chai:
    // expect(new anchor.BN(userInputTokenBalanceAfter.value.amount)).to.be.lessThan(new anchor.BN(1000000000)); // User should have less input tokens now
    // expect(new anchor.BN(userBpTokenBalanceAfter.value.amount)).to.be.greaterThan(new anchor.BN(0)); // User should have received BP tokens

  });*/


});
