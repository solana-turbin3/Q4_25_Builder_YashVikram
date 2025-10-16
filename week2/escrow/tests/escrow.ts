import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { Escrow } from "../target/types/escrow";
import { expect } from "chai";
import {createMint, getAssociatedTokenAddressSync, getOrCreateAssociatedTokenAccount, mintTo, TOKEN_PROGRAM_ID, ASSOCIATED_TOKEN_PROGRAM_ID} from "@solana/spl-token";

describe("escrow", () => {
  // Configure the client to use the local cluster.
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.Escrow as Program<Escrow>;

  const maker = provider.wallet.publicKey;
  const taker = anchor.web3.Keypair.generate();

  let mintA: anchor.web3.PublicKey;
  let mintB: anchor.web3.PublicKey;
  let makerAtaA: anchor.web3.PublicKey;
  let takerAtaB: anchor.web3.PublicKey;
  let makerAtaB: anchor.web3.PublicKey;
  let takerAtaA: anchor.web3.PublicKey;

  const seed = new anchor.BN(1234);
  let escrowPda: anchor.web3.PublicKey;
  let escrowBump: number;
  let vault: anchor.web3.PublicKey;

  const depositAmount = 100;
  const receiveAmount = 200;
  
  before(async()=> {

    // step:1 airdrop the wallets
    await provider.connection.requestAirdrop(maker, 10*anchor.web3.LAMPORTS_PER_SOL);
    await provider.connection.requestAirdrop(taker.publicKey, 10*anchor.web3.LAMPORTS_PER_SOL);

    // step:2 create the mints
    mintA = await createMint(provider.connection,provider.wallet.payer,provider.wallet.publicKey,null,0);
    mintB = await createMint(provider.connection,provider.wallet.payer,provider.wallet.publicKey,null,0);

    // step:3 create ata and mint
    makerAtaA = (await getOrCreateAssociatedTokenAccount(provider.connection, provider.wallet.payer, mintA, maker)).address;
    await mintTo(provider.connection, provider.wallet.payer, mintA, makerAtaA, provider.wallet.publicKey, 1000);

    takerAtaB = (await getOrCreateAssociatedTokenAccount(provider.connection, provider.wallet.payer, mintB, taker.publicKey)).address;
    await mintTo(provider.connection, provider.wallet.payer, mintB, takerAtaB, provider.wallet.publicKey, 1000);

    makerAtaB = (await getOrCreateAssociatedTokenAccount(provider.connection, provider.wallet.payer, mintB, maker)).address;
    takerAtaA = (await getOrCreateAssociatedTokenAccount(provider.connection, provider.wallet.payer, mintA, taker.publicKey)).address;

  })

  // test:1 we're writing a complete test to make and revoke an escrow
  it("Makes and revokes an escrow", async () => {
    const seed1 = new anchor.BN(Math.floor(Math.random() * 1000000));
    [escrowPda, escrowBump] = anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from("escrow"), maker.toBuffer(), seed1.toArrayLike(Buffer,"le",8)],
      program.programId
    );
    vault = getAssociatedTokenAddressSync(mintA, escrowPda, true);
    // getAssociatedTokenAddressSync: Computes the expected address (PDA) of the token account for a given mint and owner.

    // calling the make_offer()
    await program.methods
    .makeOffer(seed1, new anchor.BN(depositAmount), new anchor.BN(receiveAmount))
    .accountsStrict({
      maker: maker,
      mintA: mintA,
      mintB: mintB,
      makerAtaA: makerAtaA,
      escrow: escrowPda,
      escrowVault: vault,
      systemProgram: anchor.web3.SystemProgram.programId,
      tokenProgram: TOKEN_PROGRAM_ID,
      associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
    })
    .rpc();

    const escrowAccount = await program.account.escrow.fetch(escrowPda);
    expect(escrowAccount.maker.toBase58()).to.equal(maker.toBase58());
    expect(escrowAccount.mintA.toBase58()).to.equal(mintA.toBase58());
    expect(escrowAccount.mintB.toBase58()).to.equal(mintB.toBase58());
    expect(escrowAccount.receive.toNumber()).to.equal(receiveAmount);
    expect(escrowAccount.bump).to.equal(escrowBump);

    const vaultBalance = (await provider.connection.getTokenAccountBalance(vault)).value.uiAmount;
    expect(vaultBalance).to.equal(depositAmount);

    await program.methods
      .revokeOffer(seed1)
      .accountsStrict({
        maker: maker,
        mintA: mintA,
        makerAtaA: makerAtaA,
        escrow: escrowPda,
        escrowVault: vault,
        associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
        tokenProgram: TOKEN_PROGRAM_ID,
      })
      .rpc();

    // Check closed
    const escrowInfo = await provider.connection.getAccountInfo(escrowPda);
    expect(escrowInfo).to.be.null;

    const vaultInfo = await provider.connection.getAccountInfo(vault);
    expect(vaultInfo).to.be.null;
  });

  it("Makes escrow and takes the transaction", async() => {
        
    const seed2 = new anchor.BN(Math.floor(Math.random() * 1000000));
    [escrowPda, escrowBump] = anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from("escrow"),maker.toBuffer(),seed2.toArrayLike(Buffer, "le", 8)],
      program.programId
    );
    vault = getAssociatedTokenAddressSync(mintA,escrowPda,true);

    await program.methods
    .makeOffer(seed2,new anchor.BN(depositAmount), new anchor.BN(receiveAmount))
    .accountsStrict({
        maker: maker,
        mintA: mintA,
        mintB: mintB,
        makerAtaA: makerAtaA,
        escrow: escrowPda,
        escrowVault: vault,
        systemProgram: anchor.web3.SystemProgram.programId,
        tokenProgram: TOKEN_PROGRAM_ID,
        associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
    })
    .rpc();

    await program.methods
    .takeOffer(seed2)
    .accountsStrict({
        taker: taker.publicKey,
        maker: maker,
        mintA: mintA,
        mintB: mintB,
        takerAtaA: takerAtaA,
        takerAtaB: takerAtaB,
        makerAtaB: makerAtaB,
        escrow: escrowPda,
        escrowVault: vault,
        systemProgram: anchor.web3.SystemProgram.programId,
        tokenProgram: TOKEN_PROGRAM_ID,
        associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
    })
    .signers([taker])
    .rpc();

    // Check closed
    const escrowInfo = await provider.connection.getAccountInfo(escrowPda);
    expect(escrowInfo).to.be.null;

    const vaultInfo = await provider.connection.getAccountInfo(vault);
    expect(vaultInfo).to.be.null;

    // Check balances
    const takerBalanceA = (await provider.connection.getTokenAccountBalance(takerAtaA)).value.uiAmount;
    expect(takerBalanceA).to.equal(depositAmount);

    const makerBalanceB = (await provider.connection.getTokenAccountBalance(makerAtaB)).value.uiAmount;
    expect(makerBalanceB).to.equal(receiveAmount);

    // Check that taker's mintB balance decreased by receiveAmount
    const takerBalanceB = (await provider.connection.getTokenAccountBalance(takerAtaB)).value.uiAmount;
    expect(takerBalanceB).to.equal(1000 - receiveAmount);

    // Check that maker's mintA balance decreased by depositAmount (since it was transferred to escrow)
    const makerBalanceA = (await provider.connection.getTokenAccountBalance(makerAtaA)).value.uiAmount;
    expect(makerBalanceA).to.equal(1000 - depositAmount);
  })


});
