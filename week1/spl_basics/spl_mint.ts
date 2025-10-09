import { Keypair, PublicKey, Connection, Commitment } from "@solana/web3.js";
import wallet from "../../dev-wallet.json"

// Import our keypair from the wallet file
const keypair = Keypair.fromSecretKey(new Uint8Array(wallet));

//Create a Solana devnet connection
const commitment: Commitment = "confirmed";
const connection = new Connection("https://api.devnet.solana.com", commitment);

const token_decimals = 1_000_000n;

// Mint address
const mint = new PublicKey("GGac41Kr8gdrifHcwdKTxEQnW3pEbwXp8b9RTo6czTrt");

(async () => {
    try {
        // Dynamically import the ESM-only spl-token module
        const { getOrCreateAssociatedTokenAccount, mintTo } = await import('@solana/spl-token');

        // Create an ATA
        const ata = await getOrCreateAssociatedTokenAccount(connection, keypair, mint, keypair.publicKey);
        console.log(`Your ata is: ${ata.address.toBase58()}`);

        // Mint to ATA
        const mintTx = await mintTo(connection, keypair, mint, ata.address, keypair.publicKey, 1000000);
        console.log(`Your mint txid: ${mintTx}`);
    } catch(error) {
        console.log(`Oops, something went wrong: ${error}`)
    }
})()
