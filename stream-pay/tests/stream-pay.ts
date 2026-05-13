import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { StreamPay } from "../target/types/stream_pay";
import {
  createMint,
  createAccount,
  mintTo,
  getAccount,
  TOKEN_PROGRAM_ID,
} from "@solana/spl-token";
import { assert } from "chai";
import { BN } from "bn.js";

const STREAM_SEED = Buffer.from("stream");
const VAULT_SEED = Buffer.from("vault");

describe("stream_pay", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);
  const program = anchor.workspace.StreamPay as Program<StreamPay>;

  let payer: anchor.web3.Keypair;
  let recipient: anchor.web3.Keypair;
  let mint: anchor.web3.PublicKey;
  let payerTokenAccount: anchor.web3.PublicKey;
  let recipientTokenAccount: anchor.web3.PublicKey;
  let streamSeed: BN;
  let streamPda: anchor.web3.PublicKey;
  let vaultPda: anchor.web3.PublicKey;
  let vaultAuthorityPda: anchor.web3.PublicKey;
  let bump: number;
  let vaultBump: number;

  const RATE = new BN(1_000_000_000); // 1 token/sec
  const DEPOSIT_AMOUNT = new BN(1_000_000_000); // 1 token

  beforeEach(async () => {
    payer = anchor.web3.Keypair.generate();
    recipient = anchor.web3.Keypair.generate();

    const airdropSig = await provider.connection.requestAirdrop(
      payer.publicKey,
      10 * anchor.web3.LAMPORTS_PER_SOL
    );
    await provider.connection.confirmTransaction(airdropSig);

    mint = await createMint(
      provider.connection,
      payer,
      payer.publicKey,
      null,
      9
    );

    payerTokenAccount = await createAccount(
      provider.connection,
      payer,
      mint,
      payer.publicKey
    );

    recipientTokenAccount = await createAccount(
      provider.connection,
      payer,
      mint,
      recipient.publicKey
    );

    await mintTo(
      provider.connection,
      payer,
      mint,
      payerTokenAccount,
      payer.publicKey,
      10_000_000_000 // 10 tokens
    );

    streamSeed = new BN(Math.floor(Math.random() * 1_000_000));

    [streamPda, bump] = anchor.web3.PublicKey.findProgramAddressSync(
      [STREAM_SEED, payer.publicKey.toBuffer(), streamSeed.toArrayLike(Buffer, "le", 8)],
      program.programId
    );

    [vaultPda, vaultBump] = anchor.web3.PublicKey.findProgramAddressSync(
      [VAULT_SEED, streamPda.toBuffer()],
      program.programId
    );

    [vaultAuthorityPda] = anchor.web3.PublicKey.findProgramAddressSync(
      [VAULT_SEED, streamPda.toBuffer()],
      program.programId
    );
  });

  // ──────────────────────────────────────────
  // create_stream
  // ──────────────────────────────────────────
  describe("create_stream", () => {
    it("creates a stream successfully", async () => {
      const now = Math.floor(Date.now() / 1000);
      const startTime = new BN(now + 10);
      const endTime = new BN(now + 100);

      await program.methods
        .createStream(streamSeed, DEPOSIT_AMOUNT, RATE, startTime, endTime)
        .accounts({
          stream: streamPda,
          vault: vaultPda,
          vaultAuthority: vaultAuthorityPda,
          payerTokenAccount: payerTokenAccount,
          recipient: recipient.publicKey,
          payer: payer.publicKey,
          mint: mint,
          tokenProgram: TOKEN_PROGRAM_ID,
          systemProgram: anchor.web3.SystemProgram.programId,
          rent: anchor.web3.SYSVAR_RENT_PUBKEY,
        })
        .signers([payer])
        .rpc();

      const stream = await program.account.stream.fetch(streamPda);
      assert.strictEqual(stream.payer.toString(), payer.publicKey.toString());
      assert.strictEqual(stream.recipient.toString(), recipient.publicKey.toString());
      assert.strictEqual(stream.mint.toString(), mint.toString());
      assert.isTrue(stream.rate.eq(RATE));
      assert.isTrue(stream.totalAmount.eq(DEPOSIT_AMOUNT));
      assert.isTrue(stream.withdrawnAmount.eq(new BN(0)));
      assert.isTrue(stream.startTime.eq(startTime));
      assert.isTrue(stream.endTime.eq(endTime));
      assert.strictEqual(stream.pausedAt.toNumber(), 0);
      assert.isTrue(stream.seed.eq(streamSeed));
      assert.strictEqual(stream.bump, bump);
      assert.strictEqual(stream.vaultBump, vaultBump);

      const vaultAccount = await getAccount(provider.connection, vaultPda);
      assert.strictEqual(vaultAccount.amount.toString(), DEPOSIT_AMOUNT.toString());
    });

    it("supports open-ended duration (end_time = 0)", async () => {
      const now = Math.floor(Date.now() / 1000);
      const startTime = new BN(now + 10);
      const endTime = new BN(0);

      await program.methods
        .createStream(streamSeed, DEPOSIT_AMOUNT, RATE, startTime, endTime)
        .accounts({
          stream: streamPda,
          vault: vaultPda,
          vaultAuthority: vaultAuthorityPda,
          payerTokenAccount: payerTokenAccount,
          recipient: recipient.publicKey,
          payer: payer.publicKey,
          mint: mint,
          tokenProgram: TOKEN_PROGRAM_ID,
          systemProgram: anchor.web3.SystemProgram.programId,
          rent: anchor.web3.SYSVAR_RENT_PUBKEY,
        })
        .signers([payer])
        .rpc();

      const stream = await program.account.stream.fetch(streamPda);
      assert.strictEqual(stream.endTime.toNumber(), 0);
    });

    it("rejects amount = 0", async () => {
      const now = Math.floor(Date.now() / 1000);
      try {
        await program.methods
          .createStream(streamSeed, new BN(0), RATE, new BN(now + 10), new BN(now + 100))
          .accounts({
            stream: streamPda,
            vault: vaultPda,
            vaultAuthority: vaultAuthorityPda,
            payerTokenAccount: payerTokenAccount,
            recipient: recipient.publicKey,
            payer: payer.publicKey,
            mint: mint,
            tokenProgram: TOKEN_PROGRAM_ID,
            systemProgram: anchor.web3.SystemProgram.programId,
            rent: anchor.web3.SYSVAR_RENT_PUBKEY,
          })
          .signers([payer])
          .rpc();
        assert.fail("should have thrown");
      } catch (err) {
        const msg = err.toString();
        assert.isTrue(msg.includes("Invalid amount") || msg.includes("6000"));
      }
    });

    it("rejects rate = 0", async () => {
      const now = Math.floor(Date.now() / 1000);
      try {
        await program.methods
          .createStream(streamSeed, DEPOSIT_AMOUNT, new BN(0), new BN(now + 10), new BN(now + 100))
          .accounts({
            stream: streamPda,
            vault: vaultPda,
            vaultAuthority: vaultAuthorityPda,
            payerTokenAccount: payerTokenAccount,
            recipient: recipient.publicKey,
            payer: payer.publicKey,
            mint: mint,
            tokenProgram: TOKEN_PROGRAM_ID,
            systemProgram: anchor.web3.SystemProgram.programId,
            rent: anchor.web3.SYSVAR_RENT_PUBKEY,
          })
          .signers([payer])
          .rpc();
        assert.fail("should have thrown");
      } catch (err) {
        const msg = err.toString();
        assert.isTrue(msg.includes("Invalid rate") || msg.includes("6009"));
      }
    });

    it("rejects past start_time", async () => {
      const now = Math.floor(Date.now() / 1000);
      try {
        await program.methods
          .createStream(streamSeed, DEPOSIT_AMOUNT, RATE, new BN(now - 60), new BN(now + 100))
          .accounts({
            stream: streamPda,
            vault: vaultPda,
            vaultAuthority: vaultAuthorityPda,
            payerTokenAccount: payerTokenAccount,
            recipient: recipient.publicKey,
            payer: payer.publicKey,
            mint: mint,
            tokenProgram: TOKEN_PROGRAM_ID,
            systemProgram: anchor.web3.SystemProgram.programId,
            rent: anchor.web3.SYSVAR_RENT_PUBKEY,
          })
          .signers([payer])
          .rpc();
        assert.fail("should have thrown");
      } catch (err) {
        const msg = err.toString();
        assert.isTrue(msg.includes("Invalid timestamp") || msg.includes("6008"));
      }
    });
  });

  // ──────────────────────────────────────────
  // withdraw
  // ──────────────────────────────────────────
  describe("withdraw", () => {
    it("withdraws streamed tokens", async () => {
      const now = Math.floor(Date.now() / 1000);
      const rate = new BN(100_000_000); // 0.1 token/sec

      await program.methods
        .createStream(streamSeed, DEPOSIT_AMOUNT, rate, new BN(now + 2), new BN(now + 200))
        .accounts({
          stream: streamPda,
          vault: vaultPda,
          vaultAuthority: vaultAuthorityPda,
          payerTokenAccount: payerTokenAccount,
          recipient: recipient.publicKey,
          payer: payer.publicKey,
          mint: mint,
          tokenProgram: TOKEN_PROGRAM_ID,
          systemProgram: anchor.web3.SystemProgram.programId,
          rent: anchor.web3.SYSVAR_RENT_PUBKEY,
        })
        .signers([payer])
        .rpc();

      await new Promise((r) => setTimeout(r, 3000));

      await program.methods
        .withdraw(streamSeed, new BN(0))
        .accounts({
          stream: streamPda,
          vault: vaultPda,
          vaultAuthority: vaultAuthorityPda,
          recipientTokenAccount: recipientTokenAccount,
          payer: payer.publicKey,
          recipient: recipient.publicKey,
          tokenProgram: TOKEN_PROGRAM_ID,
        })
        .signers([recipient])
        .rpc();

      const stream = await program.account.stream.fetch(streamPda);
      assert.isTrue(stream.withdrawnAmount.gt(new BN(0)));

      const recipientAcc = await getAccount(provider.connection, recipientTokenAccount);
      assert.isTrue(recipientAcc.amount > 0n);
    });

    it("withdraws partial amount", async () => {
      const now = Math.floor(Date.now() / 1000);
      const totalAmount = new BN(10_000_000_000);

      await program.methods
        .createStream(streamSeed, totalAmount, RATE, new BN(now + 2), new BN(now + 200))
        .accounts({
          stream: streamPda,
          vault: vaultPda,
          vaultAuthority: vaultAuthorityPda,
          payerTokenAccount: payerTokenAccount,
          recipient: recipient.publicKey,
          payer: payer.publicKey,
          mint: mint,
          tokenProgram: TOKEN_PROGRAM_ID,
          systemProgram: anchor.web3.SystemProgram.programId,
          rent: anchor.web3.SYSVAR_RENT_PUBKEY,
        })
        .signers([payer])
        .rpc();

      await new Promise((r) => setTimeout(r, 3000));

      const partial = new BN(500_000_000);
      await program.methods
        .withdraw(streamSeed, partial)
        .accounts({
          stream: streamPda,
          vault: vaultPda,
          vaultAuthority: vaultAuthorityPda,
          recipientTokenAccount: recipientTokenAccount,
          payer: payer.publicKey,
          recipient: recipient.publicKey,
          tokenProgram: TOKEN_PROGRAM_ID,
        })
        .signers([recipient])
        .rpc();

      const stream = await program.account.stream.fetch(streamPda);
      assert.isTrue(stream.withdrawnAmount.eq(partial));
    });

    it("rejects when stream has not started", async () => {
      const now = Math.floor(Date.now() / 1000);
      await program.methods
        .createStream(streamSeed, DEPOSIT_AMOUNT, RATE, new BN(now + 3600), new BN(now + 7200))
        .accounts({
          stream: streamPda,
          vault: vaultPda,
          vaultAuthority: vaultAuthorityPda,
          payerTokenAccount: payerTokenAccount,
          recipient: recipient.publicKey,
          payer: payer.publicKey,
          mint: mint,
          tokenProgram: TOKEN_PROGRAM_ID,
          systemProgram: anchor.web3.SystemProgram.programId,
          rent: anchor.web3.SYSVAR_RENT_PUBKEY,
        })
        .signers([payer])
        .rpc();

      try {
        await program.methods
          .withdraw(streamSeed, new BN(0))
          .accounts({
            stream: streamPda,
            vault: vaultPda,
            vaultAuthority: vaultAuthorityPda,
            recipientTokenAccount: recipientTokenAccount,
            payer: payer.publicKey,
            recipient: recipient.publicKey,
            tokenProgram: TOKEN_PROGRAM_ID,
          })
          .signers([recipient])
          .rpc();
        assert.fail("should have thrown");
      } catch (err) {
        const msg = err.toString();
        assert.isTrue(msg.includes("not started") || msg.includes("6003"));
      }
    });

    it("rejects when non-recipient calls withdraw", async () => {
      const now = Math.floor(Date.now() / 1000);
      await program.methods
        .createStream(streamSeed, DEPOSIT_AMOUNT, RATE, new BN(now + 2), new BN(now + 200))
        .accounts({
          stream: streamPda,
          vault: vaultPda,
          vaultAuthority: vaultAuthorityPda,
          payerTokenAccount: payerTokenAccount,
          recipient: recipient.publicKey,
          payer: payer.publicKey,
          mint: mint,
          tokenProgram: TOKEN_PROGRAM_ID,
          systemProgram: anchor.web3.SystemProgram.programId,
          rent: anchor.web3.SYSVAR_RENT_PUBKEY,
        })
        .signers([payer])
        .rpc();

      await new Promise((r) => setTimeout(r, 3000));

      try {
        await program.methods
          .withdraw(streamSeed, new BN(0))
          .accounts({
            stream: streamPda,
            vault: vaultPda,
            vaultAuthority: vaultAuthorityPda,
            recipientTokenAccount: payerTokenAccount,
            payer: payer.publicKey,
            recipient: payer.publicKey,
            tokenProgram: TOKEN_PROGRAM_ID,
          })
          .signers([payer])
          .rpc();
        assert.fail("should have thrown");
      } catch (err) {
        const msg = err.toString();
        assert.isTrue(
          msg.includes("Unauthorized") || msg.includes("6006") ||
          msg.includes("ConstraintHasOne") || msg.includes("constraint")
        );
      }
    });
  });

  // ──────────────────────────────────────────
  // pause_stream
  // ──────────────────────────────────────────
  describe("pause_stream", () => {
    beforeEach(async () => {
      const now = Math.floor(Date.now() / 1000);
      await program.methods
        .createStream(streamSeed, DEPOSIT_AMOUNT, RATE, new BN(now + 2), new BN(now + 200))
        .accounts({
          stream: streamPda,
          vault: vaultPda,
          vaultAuthority: vaultAuthorityPda,
          payerTokenAccount: payerTokenAccount,
          recipient: recipient.publicKey,
          payer: payer.publicKey,
          mint: mint,
          tokenProgram: TOKEN_PROGRAM_ID,
          systemProgram: anchor.web3.SystemProgram.programId,
          rent: anchor.web3.SYSVAR_RENT_PUBKEY,
        })
        .signers([payer])
        .rpc();
      await new Promise((r) => setTimeout(r, 3000));
    });

    it("pauses the stream", async () => {
      await program.methods
        .pauseStream(streamSeed)
        .accounts({
          stream: streamPda,
          payer: payer.publicKey,
        })
        .signers([payer])
        .rpc();

      const stream = await program.account.stream.fetch(streamPda);
      assert.isTrue(stream.pausedAt.gt(new BN(0)));
    });

    it("rejects double pause", async () => {
      await program.methods
        .pauseStream(streamSeed)
        .accounts({ stream: streamPda, payer: payer.publicKey })
        .signers([payer])
        .rpc();

      try {
        await program.methods
          .pauseStream(streamSeed)
          .accounts({ stream: streamPda, payer: payer.publicKey })
          .signers([payer])
          .rpc();
        assert.fail("should have thrown");
      } catch (err) {
        assert.isTrue(err.toString().includes("paused") || err.toString().includes("6011"));
      }
    });

    it("rejects pause by non-payer", async () => {
      try {
        await program.methods
          .pauseStream(streamSeed)
          .accounts({ stream: streamPda, payer: recipient.publicKey })
          .signers([recipient])
          .rpc();
        assert.fail("should have thrown");
      } catch (err) {
        const msg = err.toString();
        assert.isTrue(
          msg.includes("Unauthorized") || msg.includes("6006") ||
          msg.includes("ConstraintHasOne") || msg.includes("constraint")
        );
      }
    });
  });

  // ──────────────────────────────────────────
  // resume_stream
  // ──────────────────────────────────────────
  describe("resume_stream", () => {
    beforeEach(async () => {
      const now = Math.floor(Date.now() / 1000);
      await program.methods
        .createStream(streamSeed, DEPOSIT_AMOUNT, RATE, new BN(now + 1), new BN(now + 200))
        .accounts({
          stream: streamPda,
          vault: vaultPda,
          vaultAuthority: vaultAuthorityPda,
          payerTokenAccount: payerTokenAccount,
          recipient: recipient.publicKey,
          payer: payer.publicKey,
          mint: mint,
          tokenProgram: TOKEN_PROGRAM_ID,
          systemProgram: anchor.web3.SystemProgram.programId,
          rent: anchor.web3.SYSVAR_RENT_PUBKEY,
        })
        .signers([payer])
        .rpc();
      await new Promise((r) => setTimeout(r, 2000));
      await program.methods
        .pauseStream(streamSeed)
        .accounts({ stream: streamPda, payer: payer.publicKey })
        .signers([payer])
        .rpc();
    });

    it("resumes and compensates pause duration", async () => {
      const before = await program.account.stream.fetch(streamPda);
      await new Promise((r) => setTimeout(r, 2000));

      await program.methods
        .resumeStream(streamSeed)
        .accounts({ stream: streamPda, payer: payer.publicKey })
        .signers([payer])
        .rpc();

      const after = await program.account.stream.fetch(streamPda);
      assert.strictEqual(after.pausedAt.toNumber(), 0);
      assert.isTrue(after.startTime.gt(before.startTime));
    });

    it("rejects resume when not paused", async () => {
      await program.methods
        .resumeStream(streamSeed)
        .accounts({ stream: streamPda, payer: payer.publicKey })
        .signers([payer])
        .rpc();

      try {
        await program.methods
          .resumeStream(streamSeed)
          .accounts({ stream: streamPda, payer: payer.publicKey })
          .signers([payer])
          .rpc();
        assert.fail("should have thrown");
      } catch (err) {
        assert.isTrue(err.toString().includes("not paused") || err.toString().includes("6012"));
      }
    });

    it("rejects resume by non-payer", async () => {
      try {
        await program.methods
          .resumeStream(streamSeed)
          .accounts({ stream: streamPda, payer: recipient.publicKey })
          .signers([recipient])
          .rpc();
        assert.fail("should have thrown");
      } catch (err) {
        const msg = err.toString();
        assert.isTrue(
          msg.includes("Unauthorized") || msg.includes("6006") ||
          msg.includes("ConstraintHasOne") || msg.includes("constraint")
        );
      }
    });
  });

  // ──────────────────────────────────────────
  // adjust_rate
  // ──────────────────────────────────────────
  describe("adjust_rate", () => {
    beforeEach(async () => {
      const now = Math.floor(Date.now() / 1000);
      await program.methods
        .createStream(streamSeed, DEPOSIT_AMOUNT, RATE, new BN(now + 2), new BN(now + 200))
        .accounts({
          stream: streamPda,
          vault: vaultPda,
          vaultAuthority: vaultAuthorityPda,
          payerTokenAccount: payerTokenAccount,
          recipient: recipient.publicKey,
          payer: payer.publicKey,
          mint: mint,
          tokenProgram: TOKEN_PROGRAM_ID,
          systemProgram: anchor.web3.SystemProgram.programId,
          rent: anchor.web3.SYSVAR_RENT_PUBKEY,
        })
        .signers([payer])
        .rpc();
    });

    it("adjusts rate successfully", async () => {
      const newRate = new BN(2_000_000_000);
      await program.methods
        .adjustRate(streamSeed, newRate)
        .accounts({ stream: streamPda, payer: payer.publicKey })
        .signers([payer])
        .rpc();

      const stream = await program.account.stream.fetch(streamPda);
      assert.isTrue(stream.rate.eq(newRate));
    });

    it("rejects rate = 0", async () => {
      try {
        await program.methods
          .adjustRate(streamSeed, new BN(0))
          .accounts({ stream: streamPda, payer: payer.publicKey })
          .signers([payer])
          .rpc();
        assert.fail("should have thrown");
      } catch (err) {
        assert.isTrue(
          err.toString().includes("Invalid rate") || err.toString().includes("6009")
        );
      }
    });

    it("rejects adjust by non-payer", async () => {
      try {
        await program.methods
          .adjustRate(streamSeed, new BN(500_000_000))
          .accounts({ stream: streamPda, payer: recipient.publicKey })
          .signers([recipient])
          .rpc();
        assert.fail("should have thrown");
      } catch (err) {
        const msg = err.toString();
        assert.isTrue(
          msg.includes("Unauthorized") || msg.includes("6006") ||
          msg.includes("ConstraintHasOne") || msg.includes("constraint")
        );
      }
    });
  });

  // ──────────────────────────────────────────
  // close_stream
  // ──────────────────────────────────────────
  describe("close_stream", () => {
    it("closes, pays recipient, and refunds payer", async () => {
      const now = Math.floor(Date.now() / 1000);
      const totalAmount = new BN(10_000_000_000);

      await program.methods
        .createStream(streamSeed, totalAmount, RATE, new BN(now + 2), new BN(now + 200))
        .accounts({
          stream: streamPda,
          vault: vaultPda,
          vaultAuthority: vaultAuthorityPda,
          payerTokenAccount: payerTokenAccount,
          recipient: recipient.publicKey,
          payer: payer.publicKey,
          mint: mint,
          tokenProgram: TOKEN_PROGRAM_ID,
          systemProgram: anchor.web3.SystemProgram.programId,
          rent: anchor.web3.SYSVAR_RENT_PUBKEY,
        })
        .signers([payer])
        .rpc();

      await new Promise((r) => setTimeout(r, 3000));

      const rBefore = await getAccount(provider.connection, recipientTokenAccount);

      await program.methods
        .closeStream(streamSeed)
        .accounts({
          stream: streamPda,
          vault: vaultPda,
          vaultAuthority: vaultAuthorityPda,
          payerTokenAccount: payerTokenAccount,
          recipientTokenAccount: recipientTokenAccount,
          payer: payer.publicKey,
          tokenProgram: TOKEN_PROGRAM_ID,
        })
        .signers([payer])
        .rpc();

      // Stream account must be gone
      try {
        await program.account.stream.fetch(streamPda);
        assert.fail("stream should be closed");
      } catch { /* expected */ }

      // Recipient got tokens
      const rAfter = await getAccount(provider.connection, recipientTokenAccount);
      assert.isTrue(rAfter.amount > rBefore.amount);
    });

    it("closes paused stream correctly", async () => {
      const now = Math.floor(Date.now() / 1000);
      const totalAmount = new BN(10_000_000_000);

      await program.methods
        .createStream(streamSeed, totalAmount, RATE, new BN(now + 1), new BN(now + 200))
        .accounts({
          stream: streamPda,
          vault: vaultPda,
          vaultAuthority: vaultAuthorityPda,
          payerTokenAccount: payerTokenAccount,
          recipient: recipient.publicKey,
          payer: payer.publicKey,
          mint: mint,
          tokenProgram: TOKEN_PROGRAM_ID,
          systemProgram: anchor.web3.SystemProgram.programId,
          rent: anchor.web3.SYSVAR_RENT_PUBKEY,
        })
        .signers([payer])
        .rpc();

      await new Promise((r) => setTimeout(r, 3000));

      await program.methods
        .pauseStream(streamSeed)
        .accounts({ stream: streamPda, payer: payer.publicKey })
        .signers([payer])
        .rpc();

      await new Promise((r) => setTimeout(r, 2000));

      await program.methods
        .closeStream(streamSeed)
        .accounts({
          stream: streamPda,
          vault: vaultPda,
          vaultAuthority: vaultAuthorityPda,
          payerTokenAccount: payerTokenAccount,
          recipientTokenAccount: recipientTokenAccount,
          payer: payer.publicKey,
          tokenProgram: TOKEN_PROGRAM_ID,
        })
        .signers([payer])
        .rpc();

      try {
        await program.account.stream.fetch(streamPda);
        assert.fail("stream should be closed");
      } catch { /* expected */ }
    });

    it("rejects close by non-payer", async () => {
      const now = Math.floor(Date.now() / 1000);
      await program.methods
        .createStream(streamSeed, DEPOSIT_AMOUNT, RATE, new BN(now + 2), new BN(now + 200))
        .accounts({
          stream: streamPda,
          vault: vaultPda,
          vaultAuthority: vaultAuthorityPda,
          payerTokenAccount: payerTokenAccount,
          recipient: recipient.publicKey,
          payer: payer.publicKey,
          mint: mint,
          tokenProgram: TOKEN_PROGRAM_ID,
          systemProgram: anchor.web3.SystemProgram.programId,
          rent: anchor.web3.SYSVAR_RENT_PUBKEY,
        })
        .signers([payer])
        .rpc();

      await new Promise((r) => setTimeout(r, 3000));

      try {
        await program.methods
          .closeStream(streamSeed)
          .accounts({
            stream: streamPda,
            vault: vaultPda,
            vaultAuthority: vaultAuthorityPda,
            payerTokenAccount: payerTokenAccount,
            recipientTokenAccount: recipientTokenAccount,
            payer: recipient.publicKey,
            tokenProgram: TOKEN_PROGRAM_ID,
          })
          .signers([recipient])
          .rpc();
        assert.fail("should have thrown");
      } catch (err) {
        const msg = err.toString();
        assert.isTrue(
          msg.includes("Unauthorized") || msg.includes("6006") ||
          msg.includes("ConstraintHasOne") || msg.includes("constraint")
        );
      }
    });
  });

  // ──────────────────────────────────────────
  // Full lifecycle
  // ──────────────────────────────────────────
  describe("full lifecycle", () => {
    it("create → pause → withdraw-rejected → resume → withdraw → close", async () => {
      const now = Math.floor(Date.now() / 1000);
      const totalAmount = new BN(10_000_000_000);

      await program.methods
        .createStream(streamSeed, totalAmount, RATE, new BN(now + 1), new BN(now + 200))
        .accounts({
          stream: streamPda,
          vault: vaultPda,
          vaultAuthority: vaultAuthorityPda,
          payerTokenAccount: payerTokenAccount,
          recipient: recipient.publicKey,
          payer: payer.publicKey,
          mint: mint,
          tokenProgram: TOKEN_PROGRAM_ID,
          systemProgram: anchor.web3.SystemProgram.programId,
          rent: anchor.web3.SYSVAR_RENT_PUBKEY,
        })
        .signers([payer])
        .rpc();

      await new Promise((r) => setTimeout(r, 2500));

      // pause
      await program.methods
        .pauseStream(streamSeed)
        .accounts({ stream: streamPda, payer: payer.publicKey })
        .signers([payer])
        .rpc();

      let stream = await program.account.stream.fetch(streamPda);
      assert.isTrue(stream.pausedAt.gt(new BN(0)));

      // withdraw should fail while paused
      try {
        await program.methods
          .withdraw(streamSeed, new BN(0))
          .accounts({
            stream: streamPda,
            vault: vaultPda,
            vaultAuthority: vaultAuthorityPda,
            recipientTokenAccount: recipientTokenAccount,
            payer: payer.publicKey,
            recipient: recipient.publicKey,
            tokenProgram: TOKEN_PROGRAM_ID,
          })
          .signers([recipient])
          .rpc();
        assert.fail("should have thrown");
      } catch (err) {
        assert.isTrue(err.toString().includes("paused") || err.toString().includes("6011"));
      }

      await new Promise((r) => setTimeout(r, 2000));

      // resume
      await program.methods
        .resumeStream(streamSeed)
        .accounts({ stream: streamPda, payer: payer.publicKey })
        .signers([payer])
        .rpc();

      stream = await program.account.stream.fetch(streamPda);
      assert.strictEqual(stream.pausedAt.toNumber(), 0);

      await new Promise((r) => setTimeout(r, 2500));

      // withdraw
      await program.methods
        .withdraw(streamSeed, new BN(0))
        .accounts({
          stream: streamPda,
          vault: vaultPda,
          vaultAuthority: vaultAuthorityPda,
          recipientTokenAccount: recipientTokenAccount,
          payer: payer.publicKey,
          recipient: recipient.publicKey,
          tokenProgram: TOKEN_PROGRAM_ID,
        })
        .signers([recipient])
        .rpc();

      stream = await program.account.stream.fetch(streamPda);
      assert.isTrue(stream.withdrawnAmount.gt(new BN(0)));

      // close
      await program.methods
        .closeStream(streamSeed)
        .accounts({
          stream: streamPda,
          vault: vaultPda,
          vaultAuthority: vaultAuthorityPda,
          payerTokenAccount: payerTokenAccount,
          recipientTokenAccount: recipientTokenAccount,
          payer: payer.publicKey,
          tokenProgram: TOKEN_PROGRAM_ID,
        })
        .signers([payer])
        .rpc();

      try {
        await program.account.stream.fetch(streamPda);
        assert.fail("stream should be closed");
      } catch { /* expected */ }
    });
  });

  // ──────────────────────────────────────────
  // Multiple streams per pair
  // ──────────────────────────────────────────
  describe("multiple streams", () => {
    it("allows same payer+recipient with different seeds", async () => {
      const seed1 = new BN(1);
      const seed2 = new BN(2);
      const now = Math.floor(Date.now() / 1000);

      const [s1] = anchor.web3.PublicKey.findProgramAddressSync(
        [STREAM_SEED, payer.publicKey.toBuffer(), seed1.toArrayLike(Buffer, "le", 8)],
        program.programId
      );
      const [v1] = anchor.web3.PublicKey.findProgramAddressSync(
        [VAULT_SEED, s1.toBuffer()], program.programId
      );
      const [va1] = anchor.web3.PublicKey.findProgramAddressSync(
        [VAULT_SEED, s1.toBuffer()], program.programId
      );
      const [s2] = anchor.web3.PublicKey.findProgramAddressSync(
        [STREAM_SEED, payer.publicKey.toBuffer(), seed2.toArrayLike(Buffer, "le", 8)],
        program.programId
      );
      const [v2] = anchor.web3.PublicKey.findProgramAddressSync(
        [VAULT_SEED, s2.toBuffer()], program.programId
      );
      const [va2] = anchor.web3.PublicKey.findProgramAddressSync(
        [VAULT_SEED, s2.toBuffer()], program.programId
      );

      const createAccounts = (s: anchor.web3.PublicKey, v: anchor.web3.PublicKey, va: anchor.web3.PublicKey) => ({
        stream: s,
        vault: v,
        vaultAuthority: va,
        payerTokenAccount: payerTokenAccount,
        recipient: recipient.publicKey,
        payer: payer.publicKey,
        mint: mint,
        tokenProgram: TOKEN_PROGRAM_ID,
        systemProgram: anchor.web3.SystemProgram.programId,
        rent: anchor.web3.SYSVAR_RENT_PUBKEY,
      });

      await program.methods.createStream(seed1, DEPOSIT_AMOUNT, RATE, new BN(now + 2), new BN(now + 200))
        .accounts(createAccounts(s1, v1, va1)).signers([payer]).rpc();

      await program.methods.createStream(seed2, DEPOSIT_AMOUNT, RATE, new BN(now + 2), new BN(now + 200))
        .accounts(createAccounts(s2, v2, va2)).signers([payer]).rpc();

      const stream1 = await program.account.stream.fetch(s1);
      const stream2 = await program.account.stream.fetch(s2);
      assert.isTrue(stream1.seed.eq(seed1));
      assert.isTrue(stream2.seed.eq(seed2));
      assert.notStrictEqual(s1.toString(), s2.toString());
    });
  });
});
