import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { DecentralizedLendingPlatoform } from "../target/types/decentralized_lending_platoform";
import { PublicKey, SystemProgram } from "@solana/web3.js";
import {createMint, getOrCreateAssociatedTokenAccount, mintTo, TOKEN_PROGRAM_ID} from "@solana/spl-token";
import { ASSOCIATED_PROGRAM_ID } from "@coral-xyz/anchor/dist/cjs/utils/token";

const provider = anchor.AnchorProvider.env();
anchor.setProvider(provider);

const program = anchor.workspace.decentralizedLendingPlatoform as Program<DecentralizedLendingPlatoform>;

let tokenMintA: PublicKey;
let tokenMintB: PublicKey;
let liquidityPool: PublicKey;
let lpTokenMint: PublicKey;
let tokenVaultA: PublicKey;
let tokenVaultB: PublicKey;
let feeVaultA: PublicKey;
let feeVaultB: PublicKey;

let liquidityProviderAccount: PublicKey;
let providerTokenAata: PublicKey;
let providerTokenBata: PublicKey;
let providerLpTokenAta: PublicKey;

let borrowerAccountInfo: PublicKey;
let borrowerAta: PublicKey;
let borrowerCollateralAta: PublicKey;

before(async () => {
  tokenMintA = await createMint(
    provider.connection,
    provider.wallet.payer,
    provider.wallet.publicKey,
    null,
    6
  );

  tokenMintB = await createMint(
    provider.connection,
    provider.wallet.payer,
    provider.wallet.publicKey,
    null,
    6
  );

  [liquidityPool] = PublicKey.findProgramAddressSync(
    [Buffer.from("liquidity_pool"), tokenMintA.toBuffer(), tokenMintB.toBuffer(), provider.wallet.publicKey.toBuffer()],
    program.programId
  );

  [lpTokenMint] = PublicKey.findProgramAddressSync(
    [Buffer.from("lp_token_mint"), liquidityPool.toBuffer()],
    program.programId
  );

  [tokenVaultA] = PublicKey.findProgramAddressSync(
    [Buffer.from("token_vault_a"), tokenMintA.toBuffer(), liquidityPool.toBuffer()],
    program.programId
  );
  
  [tokenVaultB] = PublicKey.findProgramAddressSync(
    [Buffer.from("token_vault_b"), tokenMintB.toBuffer(), liquidityPool.toBuffer()],
    program.programId
  );

  [feeVaultA] = PublicKey.findProgramAddressSync(
    [Buffer.from("fee_vault_a"), tokenMintA.toBuffer(), liquidityPool.toBuffer()],
    program.programId
  );

  [feeVaultB] = PublicKey.findProgramAddressSync(
    [Buffer.from("fee_vault_b"), tokenMintB.toBuffer(), liquidityPool.toBuffer()],
    program.programId
  );

  [liquidityProviderAccount] = PublicKey.findProgramAddressSync(
    [Buffer.from("liquidity_provider"), provider.wallet.publicKey.toBuffer()],
    program.programId
  );

  let TokenAata = await getOrCreateAssociatedTokenAccount(
    provider.connection,
    provider.wallet.payer,
    tokenMintA,
    provider.wallet.publicKey,
  );

  providerTokenAata = TokenAata.address;

  let TokenBata = await getOrCreateAssociatedTokenAccount(
    provider.connection,
    provider.wallet.payer,
    tokenMintB,
    provider.wallet.publicKey,
  );

  providerTokenBata = TokenBata.address;

  [borrowerAccountInfo] = PublicKey.findProgramAddressSync(
    [Buffer.from("borrower_account"), provider.wallet.publicKey.toBuffer()],
    program.programId
  );

  let borrowerAtaAccount = await getOrCreateAssociatedTokenAccount(
    provider.connection,
    provider.wallet.payer,
    tokenMintA,
    provider.wallet.publicKey
  );

  borrowerAta = borrowerAtaAccount.address;

  let borrowerCollateralAccount = await getOrCreateAssociatedTokenAccount(
    provider.connection,
    provider.wallet.payer,
    tokenMintB,
    provider.wallet.publicKey
  );

  borrowerCollateralAta = borrowerCollateralAccount.address;
})

describe("decentralized_lending_platoform", () => {

  it("Is initialized!", async () => {
    // Add your test here.
    const tx = await program.methods.initialize().rpc();
    console.log("Your transaction signature", tx);
  });

  it("Initialize liquidity pool", async () => {
    let ltvRatio = 80;
    let liquidationThreshold = 85;
    let liquidationPenalty = 5;
    let interestRate = 3;

    const tx = await program.methods.initializeLiquidityPool(ltvRatio, liquidationThreshold, liquidationPenalty, interestRate).accountsPartial({
      creator: provider.wallet.publicKey,
      tokenMintA,
      tokenMintB,
      liquidityPool,
      lpTokenMint,
      tokenVaultA,
      tokenVaultB,
      feeVaultA,
      feeVaultB,
      systemProgram: SystemProgram.programId,
      tokenProgram: TOKEN_PROGRAM_ID,
    }).signers([provider.wallet.payer]).rpc();

    console.log(`Transaction Signature: ${tx}`);

    // Create LP token ATA AFTER the pool is initialized
    let LpTokenAta = await getOrCreateAssociatedTokenAccount(
      provider.connection,
      provider.wallet.payer,
      lpTokenMint,
      provider.wallet.publicKey,
    );

    providerLpTokenAta = LpTokenAta.address;
  });

  it("Update pool parameters", async () => {
    let newLtvRatio = 75;
    let newLiquidationThreshold = 80;
    let newLiquidationPenalty = 3;
    let newInterestRate = 2;

    const tx = await program.methods.updatePoolParameters(newLtvRatio, newLiquidationThreshold, newLiquidationPenalty, newInterestRate).accountsPartial({
      creator: provider.wallet.publicKey,
      liquidityPool
    }).signers([provider.wallet.payer]).rpc();

    console.log(`Transaction Signature: ${tx}`);
  });

  it("Initialize liquidity provider", async () => {
    const tx = await program.methods.initializeLiquidityProvider().accountsPartial({  
      provider: provider.wallet.publicKey,
      liquidityProviderAccount,
      systemProgram: SystemProgram.programId
    }).signers([provider.wallet.payer]).rpc();

    console.log(`Transaction Signature: ${tx}`);
  });

  it("Provide liquidity", async () => {

    await mintTo(
      provider.connection,
      provider.wallet.payer,
      tokenMintA,
      providerTokenAata,
      provider.wallet.publicKey,
      200
    );

    await mintTo(
      provider.connection,
      provider.wallet.payer,
      tokenMintB,
      providerTokenBata,
      provider.wallet.publicKey,
      200
    );

    let tokenAamount = new anchor.BN(150);
    let tokenBamount = new anchor.BN(150);
    const tx = await program.methods.provideLiquidity(tokenAamount, tokenBamount).accounts({
      provider: provider.wallet.publicKey,
      tokenMintA,
      tokenMintB,
      lpTokenMint,
      liquidityPool,
      liquidityProviderAccount,
      providerTokenAata,
      providerTokenBata,
      tokenVaultA,
      tokenVaultB,
      providerLpTokenAta,
      systemProgram: SystemProgram.programId,
      tokenProgram: TOKEN_PROGRAM_ID,
      associatedTokenProgram: ASSOCIATED_PROGRAM_ID
    }).signers([provider.wallet.payer]).rpc();

    console.log(`Transaction Signature: ${tx}`);
  });

  it("Borrow funds", async () => {
    const borrowAmount = new anchor.BN(15);
    const borrowDuration = 0; 
    const tx = await program.methods.borrowFunds(borrowAmount, borrowDuration).accounts({
      borrower: provider.wallet.publicKey,
      wantedMint: tokenMintA,
      givingMint: tokenMintB,
      borrowerAccountInfo,
      liquidityPool,
      tokenVaultA,
      tokenVaultB,
      feeVaultA,  
      feeVaultB,
      borrowerAta,
      borrowerCollateralAta,
      systemProgram: SystemProgram.programId,
      tokenProgram: TOKEN_PROGRAM_ID,
      associatedTokenProgram: ASSOCIATED_PROGRAM_ID
    }).signers([provider.wallet.payer]).rpc();

    console.log(`Transaction Signture: ${tx}`);
  })

});