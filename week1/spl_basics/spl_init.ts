import { Keypair, Connection, Commitment } from "@solana/web3.js";
import wallet from "../../dev-wallet.json"

// Import our keypair from the wallet file
const keypair = Keypair.fromSecretKey(new Uint8Array(wallet));

//Create a Solana devnet connection
const commitment: Commitment = "confirmed";
const connection = new Connection("https://api.devnet.solana.com", commitment);

(async () => {
    try {
        const { createMint } = await import('@solana/spl-token');
        const mint = await createMint(connection,keypair,keypair.publicKey,null,6,)
        console.log(mint.toBase58())
    } catch(error) {
        console.log(`Oops, something went wrong: ${error}`)
    }
})()
