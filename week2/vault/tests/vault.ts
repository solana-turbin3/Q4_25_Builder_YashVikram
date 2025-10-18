import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { Vault } from "../target/types/vault";
import { expect } from "chai";

describe("vault", () => {
  // Configure the client to use the local cluster.
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.vault as Program<Vault>;

  // the instructions in vault contain four accounts: signer, vault, vaultState, system_program
  const signer = provider.wallet.publicKey;

  const [vaultStatePda, vaultStateBump] = anchor.web3.PublicKey.findProgramAddressSync(
    [Buffer.from("state"), signer.toBuffer()],
    program.programId
  );

  const [vaultPda, vaultBump] = anchor.web3.PublicKey.findProgramAddressSync(
    [Buffer.from("vault"), vaultStatePda.toBuffer()],
    program.programId
  );

  before(async()=>{
    await provider.connection.requestAirdrop(signer, 10*anchor.web3.LAMPORTS_PER_SOL);
    await new Promise(resolve => setTimeout(resolve,100));
  })

  // test:1 initialize the vault
  it("Initialize the vault", async () => {

    await program.methods
    .initialize()
    .accountsStrict({
      signer: signer,
      vault: vaultPda,
      vaultState: vaultStatePda,
      systemProgram: anchor.web3.SystemProgram.programId
    })
    .rpc()

    const vaultState = await program.account.vaultState.fetch(vaultStatePda);
    expect(vaultState.vaultBump).to.equal(vaultBump);
    expect(vaultState.stateBump).to.equal(vaultStateBump);

    const vaultBalance = await provider.connection.getBalance(vaultPda);
    const rentExempt = await provider.connection.getMinimumBalanceForRentExemption(0);

    expect(vaultBalance).to.equal(rentExempt);

  });

  // test:2 deposit tokens
  it("Deposits tokens in vault", async ()=> {

    const depositAmount = 1 * anchor.web3.LAMPORTS_PER_SOL;

    const initialVaultBalance = await provider.connection.getBalance(vaultPda);
    const initialSignerBalance = await provider.connection.getBalance(signer);
    
    const tx = await program.methods
    .deposit(new anchor.BN(depositAmount))
    .accountsStrict({
      signer: signer,
      vault: vaultPda,
      vaultState: vaultStatePda,
      systemProgram: anchor.web3.SystemProgram.programId,
    })
    .rpc();

    const afterVaultBalance = await provider.connection.getBalance(vaultPda);
    const afterSignerBalance = await provider.connection.getBalance(signer);

    // Calculate actual transaction fee
    const actualFee = initialSignerBalance - afterSignerBalance - depositAmount;

    expect(initialVaultBalance + depositAmount).to.equal(afterVaultBalance);
    expect(afterSignerBalance).to.equal(initialSignerBalance - depositAmount - actualFee);
    
  })

  it("Withdraws lamports from the vault", async()=>{

    const withdrawAmount = 0.5 * anchor.web3.LAMPORTS_PER_SOL;

    const initialSignerBalance = await provider.connection.getBalance(signer);
    const initialVaultBalance = await provider.connection.getBalance(vaultPda);

    const tx = await program.methods
    .withdraw(new anchor.BN(withdrawAmount))
    .accountsStrict({
      signer: signer,
      vault: vaultPda,
      vaultState: vaultStatePda,
      systemProgram: anchor.web3.SystemProgram.programId,
    })
    .rpc();

    const finalSignerBalance = await provider.connection.getBalance(signer);
    const finalVaultBalance = await provider.connection.getBalance(vaultPda);

    // Calculate actual transaction fee
    const actualFee = initialSignerBalance + withdrawAmount - finalSignerBalance;

    expect(initialSignerBalance + withdrawAmount - actualFee).to.equal(finalSignerBalance);
    expect(initialVaultBalance).to.equal(finalVaultBalance + withdrawAmount);

  })

  it("Closes the vault", async()=>{

    const initialVaultBalance = await provider.connection.getBalance(vaultPda);
    const initialSignerBalance = await provider.connection.getBalance(signer);
    const initialVaultStateBalance = await provider.connection.getBalance(vaultStatePda);

    const tx = await program.methods
    .closeVault()
    .accountsStrict({
      signer: signer,
      vault: vaultPda,
      vaultState: vaultStatePda,
      systemProgram: anchor.web3.SystemProgram.programId,
    })
    .rpc();

    // after closing the vault
    // 1. vault state should be null
    const vaultStateInfo = await provider.connection.getAccountInfo(vaultStatePda);
    expect(vaultStateInfo).to.be.null;

    // 2. vault should have no balance
    const finalVaultBalance = await provider.connection.getBalance(vaultPda);
    expect(finalVaultBalance).to.equal(0);

    // 3. final signer balance should be
    const finalSignerBalance = await provider.connection.getBalance(signer);

    // Calculate actual transaction fee
    const expectedFinalBalance = initialSignerBalance + initialVaultBalance + initialVaultStateBalance;
    const actualFee = expectedFinalBalance - finalSignerBalance;

    expect(finalSignerBalance).to.equal(initialSignerBalance + initialVaultBalance + initialVaultStateBalance - actualFee);
  })
});
