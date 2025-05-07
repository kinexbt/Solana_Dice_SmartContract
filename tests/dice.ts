import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { Dice } from "../target/types/dice";
import {
  PublicKey,
  SystemProgram,
  Keypair,
  Connection,
  LAMPORTS_PER_SOL,
  sendAndConfirmTransaction,
  Transaction,
} from "@solana/web3.js";
import { expect } from "chai";
import { bs58 } from "@coral-xyz/anchor/dist/cjs/utils/bytes";

let cluster = "devnet";

// Configure the client to use the local cluster.
const provider = anchor.AnchorProvider.env();
anchor.setProvider(provider);

const program = anchor.workspace.Dice as Program<Dice>;
// const admin = provider.wallet;

const superAdmin = Keypair.fromSecretKey(
  new Uint8Array([
    246, 178, 186, 25, 177, 17, 149, 237, 92, 11, 167, 16, 64, 244, 18, 204,
    160, 162, 164, 145, 146, 101, 143, 192, 167, 212, 25, 115, 34, 75, 112, 42,
    64, 216, 229, 178, 27, 23, 122, 153, 53, 64, 221, 63, 160, 125, 128, 202,
    102, 234, 103, 129, 208, 247, 237, 108, 80, 240, 39, 251, 251, 190, 254, 40,
  ])
);
console.log("Super Admin: ", superAdmin.publicKey.toBase58());

const player = Keypair.fromSecretKey(
  bs58.decode(
    "3U2WuZS8rGb4dwf9jhrhYxqvgiYNPj3Gwt4MsEeiaxsdhGrjUcQjE4YkaxDn5j9Mw9QLx6RkGA34dbitkQjLXLoe"
  )
);
// 9FRdCaJPns1ZHKYaeMgBsVvMXZDjoigmkLYFr5x1vHvt
const secondPlayer = Keypair.fromSecretKey(
  bs58.decode(
    "ANFTDi6AiwiAAe1dykZ6v6nzDiW9XPPED1XuehSQnwMqpnCq3E1Y1wT1LVncnU9T8NUdY3MTCCtnVNowyb988No"
  )
);
// 97hiuq2aRBcfPwjTecWbw8XzAQw3LRaPipK3XCx1STmV
const operationAdmin = Keypair.fromSecretKey(
  bs58.decode(
    "51JQ24MkNiHeFTo2mgopb6JmFLCRuDgbJzgRRPPeoDXkjykANew8CZaEyCncVrefNpjepYaMLjpL4R3GsgyRE7qb"
  )
);
// H8mmSFZk4K3JPjHWW3Ptrg6vM7eoZxErr7f2i1YvkNZ7
const financialAdmin = Keypair.fromSecretKey(
  bs58.decode(
    "4LL9PT3ZjUQbPEKMuuk4zhEk9jQWLh7Kz2hEqDB9baWbivdMXHg3mLre98Y78rxedmB7XDGypYCQjktDFToQBHin"
  )
);
// 6nz4SXnVsHCzLpkgA48ayR91iR6phDeF2kMoyBizSMGi
const updateAdmin = Keypair.fromSecretKey(
  bs58.decode(
    "ANFTDi6AiwiAAe1dykZ6v6nzDiW9XPPED1XuehSQnwMqpnCq3E1Y1wT1LVncnU9T8NUdY3MTCCtnVNowyb988No"
  )
);
// 97hiuq2aRBcfPwjTecWbw8XzAQw3LRaPipK3XCx1STmV

const newAdmin = Keypair.generate();
const game_session_id = new anchor.BN(1);
const second_game_session_id = new anchor.BN(2);

const connection =
  cluster == "localnet"
    ? new Connection("http://localhost:8899", "confirmed")
    : new Connection(
        "https://devnet.helius-rpc.com/?api-key=27fd6baa-75e9-4d39-9832-d5a43419ad78",
        "confirmed"
      );

let globalAuthorityPDA;
let casinoVaultPDA;
let playerPoolPDA;
let secondPlayerPoolPDA;
let gameVaultPDA;
let secondGameVaultPDA;
let playerPool;

describe("Dice Game", () => {
  before(async () => {
    if (cluster == "localnet") {
      // Airdrop SOL to test accounts
      const sig1 = await connection.requestAirdrop(
        player.publicKey,
        1000 * LAMPORTS_PER_SOL
      );
      const sig2 = await connection.requestAirdrop(
        superAdmin.publicKey,
        1000 * LAMPORTS_PER_SOL
      );
      const sig3 = await connection.requestAirdrop(
        financialAdmin.publicKey,
        1000 * LAMPORTS_PER_SOL
      );
      const sig4 = await connection.requestAirdrop(
        updateAdmin.publicKey,
        1000 * LAMPORTS_PER_SOL
      );
      const sig5 = await connection.requestAirdrop(
        operationAdmin.publicKey,
        1000 * LAMPORTS_PER_SOL
      );
      await sleep(3000);
      console.log("Airdrop Sig => ", sig5);
    }

    // Initialize program
    [globalAuthorityPDA] = await PublicKey.findProgramAddress(
      [Buffer.from("global-authority")],
      program.programId
    );
    [casinoVaultPDA] = await PublicKey.findProgramAddress(
      [Buffer.from("vault-authority")],
      program.programId
    );
    [playerPoolPDA] = await PublicKey.findProgramAddress(
      [player.publicKey.toBuffer(), Buffer.from("player-pool"), game_session_id.toArrayLike(Buffer, 'be', 8)],
      program.programId,
    );
    [secondPlayerPoolPDA] = await PublicKey.findProgramAddress(
      [secondPlayer.publicKey.toBuffer(), Buffer.from("player-pool"), second_game_session_id.toArrayLike(Buffer, 'be', 8)],
      program.programId
    );
    [gameVaultPDA] = await PublicKey.findProgramAddress(
      [player.publicKey.toBuffer(), Buffer.from("vault-authority"), game_session_id.toArrayLike(Buffer, 'be', 8)],
      program.programId
    );
    [secondGameVaultPDA] = await PublicKey.findProgramAddress(
      [secondPlayer.publicKey.toBuffer(), Buffer.from("vault-authority"), second_game_session_id.toArrayLike(Buffer, 'be', 8)],
      program.programId
    );
  });

  it("Is initialized!", async () => {
    // Add your test here.
    const tx = await program.methods
      .initialize(
        operationAdmin.publicKey,
        financialAdmin.publicKey,
        updateAdmin.publicKey
      )
      .accounts({
        admin: superAdmin.publicKey,
      })
      .signers([superAdmin])
      .rpc();

    await sleep(3000);

    const casinoTx = new Transaction().add(
      SystemProgram.transfer({
        fromPubkey: superAdmin.publicKey,
        toPubkey: casinoVaultPDA,
        lamports: 1 * LAMPORTS_PER_SOL,
      })
    );

    casinoTx.feePayer = superAdmin.publicKey;
    casinoTx.recentBlockhash = (
      await connection.getLatestBlockhash()
    ).blockhash;

    const sig = await sendAndConfirmTransaction(connection, casinoTx, [
      superAdmin,
    ]);
    console.log(`Casino vault deposit sig => https://solscan.io/${sig}`);
  });

  it("should allow a player to place a bet", async () => {
    const betAmount = new anchor.BN(0.2 * LAMPORTS_PER_SOL); // 0.1 SOL
    const targetNumber = 57;
    const is_under = true;

    console.log("betAmount: ", betAmount);

    const tx = await program.methods
      .playGame(targetNumber, is_under, betAmount, game_session_id)
      .accounts({
        owner: player.publicKey,
        operator: operationAdmin.publicKey,
        playerPool: playerPoolPDA,
        gameVault: gameVaultPDA,
      })
      .signers([player, operationAdmin])
      .transaction();
    tx.feePayer = operationAdmin.publicKey;
    tx.recentBlockhash = (await connection.getLatestBlockhash()).blockhash;
    console.log(await connection.simulateTransaction(tx));
    const sig = await sendAndConfirmTransaction(connection, tx, [
      player,
      operationAdmin,
    ]);
    console.log(`Place Bet Sig => https://solscan.io/${sig}`);

    await sleep(3000);

    console.log(
      "PDA sol balance: ",
      await connection.getBalance(playerPoolPDA)
    );
    console.log(
      "PDA AccountInfo: ",
      await connection.getAccountInfo(playerPoolPDA)
    );

    playerPool = await program.account.playerPool.fetch(playerPoolPDA);
    console.log("Player pool after place bet: ", playerPool);

    expect(playerPool.player.toString()).to.equal(player.publicKey.toString());
    expect(playerPool.bet.toString()).to.equal(betAmount.toString());
    expect(playerPool.targetNum.toString()).to.equal(targetNumber.toString());
    expect(playerPool.isUnder.toString()).to.equal(is_under.toString());
    expect(playerPool.status).to.deep.equal({ active: {} });
  });

  it("should reject bets below minimum", async () => {
    const betAmount = new anchor.BN(0.001 * LAMPORTS_PER_SOL); // Too small

    try {
      const tx = await program.methods
        .playGame(57, true, betAmount, second_game_session_id)
        .accounts({
          owner: secondPlayer.publicKey,
          operator: operationAdmin.publicKey,
          playerPool: secondPlayerPoolPDA,
          gameVault: secondGameVaultPDA,
        })
        .signers([secondPlayer, operationAdmin])
        .transaction();

      tx.feePayer = operationAdmin.publicKey;
      tx.recentBlockhash = (await connection.getLatestBlockhash()).blockhash;
      console.log(await connection.simulateTransaction(tx));
      const sig = await sendAndConfirmTransaction(connection, tx, [
        secondPlayer,
        operationAdmin,
      ]);
      console.log(`Sig => https://solscan.io/${sig}`);

      expect.fail("Should have thrown error");
    } catch (err) {
      console.log("Full error object:", JSON.stringify(err, null, 2));
      expect(err.toString()).to.contain("InvalidBetAmount");
    }
  });

  it("should process a win correctly", async () => {
    const initialBalance = await connection.getBalance(gameVaultPDA);
    console.log("Initial Game Balance before win was {}", initialBalance);

    const tx = await program.methods
      .setResult(true, game_session_id)
      .accounts({
        owner: player.publicKey,
        operator: operationAdmin.publicKey,
        gameVault: gameVaultPDA,
        playerPool: playerPoolPDA,
      })
      .transaction();
    tx.feePayer = operationAdmin.publicKey;
    tx.recentBlockhash = (await connection.getLatestBlockhash()).blockhash;
    console.log(await connection.simulateTransaction(tx));
    const sig = await sendAndConfirmTransaction(connection, tx, [
      player,
      operationAdmin,
    ]);
    console.log(`Win Sig => https://solscan.io/${sig}`);

    // const playerPool = await program.account.playerPool.fetch(playerPoolPDA);
    // console.log("PlayerPool after Win => ", playerPool);
    // expect(playerPool.status).to.deep.equal({ win: {} });

    const finalBalance = await connection.getBalance(gameVaultPDA);
    console.log("Final Game Balance after win is {}", finalBalance);
  });

  it("should process a loss correctly", async () => {
    const betAmount = new anchor.BN(0.2 * LAMPORTS_PER_SOL); // 0.2 SOL

    const tx = await program.methods
        .playGame(57, true, betAmount, second_game_session_id)
        .accounts({
          owner: secondPlayer.publicKey,
          operator: operationAdmin.publicKey,
          gameVault: secondGameVaultPDA,
          playerPool: secondPlayerPoolPDA,
        })
        .signers([secondPlayer, operationAdmin])
        .transaction();

      tx.feePayer = operationAdmin.publicKey;
      tx.recentBlockhash = (await connection.getLatestBlockhash()).blockhash;
      console.log(await connection.simulateTransaction(tx));
      const sig = await sendAndConfirmTransaction(connection, tx, [
        secondPlayer,
        operationAdmin,
      ]);
      console.log(`Second PlaceBet Sig => https://solscan.io/${sig}`);

    // Then set as loss
    const lossTx = await program.methods
      .setResult(false, second_game_session_id)
      .accounts({
        owner: secondPlayer.publicKey,
        operator: operationAdmin.publicKey,
        playerPool: secondPlayerPoolPDA,
        gameVault: secondGameVaultPDA,
      })
      .transaction();

    lossTx.feePayer = operationAdmin.publicKey;
    lossTx.recentBlockhash = (await connection.getLatestBlockhash()).blockhash;
    console.log(await connection.simulateTransaction(lossTx));
    const lossSig = await sendAndConfirmTransaction(connection, lossTx, [
      secondPlayer,
      operationAdmin,
    ]);
    console.log(`Set Loss Sig => https://solscan.io/${lossSig}`);

    // playerPool = await program.account.playerPool.fetch(playerPoolPDA);
    // console.log("PlayerPool after lose of 2nd round: ", playerPool);

    // expect(playerPool.status).to.deep.equal({ lose: {} });
  });

  it("should allow admin to withdraw funds", async () => {
    const recipient = Keypair.generate();
    const amount = new anchor.BN(0.5 * LAMPORTS_PER_SOL);

    const tx = await program.methods
      .withdraw(amount)
      .accounts({
        financialAdmin: financialAdmin.publicKey,
        recipient: recipient.publicKey,
      })
      .transaction();
    tx.feePayer = financialAdmin.publicKey;
    tx.recentBlockhash = (await connection.getLatestBlockhash()).blockhash;
    console.log(await connection.simulateTransaction(tx));
    const sig = await sendAndConfirmTransaction(connection, tx, [
      financialAdmin,
    ]);
    console.log(`RTP update Sig => https://solscan.io/${sig}`);

    const recipientBalance = await connection.getBalance(recipient.publicKey);
    expect(recipientBalance).to.equal(amount.toNumber());
  });

  it("should allow updating RTP", async () => {
    const newRtp = new anchor.BN(90);
    const tx = await program.methods
      .setRtp(newRtp)
      .accounts({
        admin: updateAdmin.publicKey,
        globalPool: globalAuthorityPDA,
      })
      .transaction();

    tx.feePayer = updateAdmin.publicKey;
    tx.recentBlockhash = (await connection.getLatestBlockhash()).blockhash;
    console.log(await connection.simulateTransaction(tx));
    const sig = await sendAndConfirmTransaction(connection, tx, [updateAdmin]);
    console.log(`RTP update Sig => https://solscan.io/${sig}`);

    const globalAuthority = await program.account.globalPool.fetch(
      globalAuthorityPDA
    );
    expect(newRtp.eq(globalAuthority.rtp)).to.be.true;
  });

  it("should allow updating authorities", async () => {
    const tx = await program.methods
      .setOperationAuthority(newAdmin.publicKey)
      .accounts({
        admin: superAdmin.publicKey,
        globalPool: globalAuthorityPDA,
      })
      .transaction();
    tx.feePayer = superAdmin.publicKey;
    tx.recentBlockhash = (await connection.getLatestBlockhash()).blockhash;
    console.log(await connection.simulateTransaction(tx));
    const sig = await sendAndConfirmTransaction(connection, tx, [superAdmin]);
    console.log(`RTP update Sig => https://solscan.io/${sig}`);

    const globalAuthority = await program.account.globalPool.fetch(
      globalAuthorityPDA
    );
    expect(globalAuthority.operationAuthority.toString()).to.equal(
      newAdmin.publicKey.toString()
    );
  });
});

function sleep(ms) {
  return new Promise((resolve) => setTimeout(resolve, ms));
}
