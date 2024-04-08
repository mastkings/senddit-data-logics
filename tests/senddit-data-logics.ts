import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { PublicKey, Keypair, SystemProgram } from '@solana/web3.js';
import { SendditDataLogics } from "../target/types/senddit_data_logics";
import assert from "assert";


describe("senddit-data-logics", () => {
  // Configure the client to use the local cluster.
  anchor.setProvider(anchor.AnchorProvider.env());
  
  const provider = anchor.AnchorProvider.env();
  const program = anchor.workspace.SendditDataLogics as Program<SendditDataLogics>;
  
  const authority = Keypair.generate();
  const treasury = Keypair.generate().publicKey;
  const ctx = {
    authority,
    treasury,
    provider,
  }
  const myPost = "https://mypost.com";
  const postUpvote = "1";
  const myComment = "my comment";
  const commentUpvote = "1";

  it("Is initialized!", async () => {
    // Add your test here.
    await program.rpc.initialize({
      accounts: {
        senddit: authority.publicKey,
        authority: authority.publicKey,
        systemProgram: SystemProgram.programId,
      },
      signers: [authority],
    });

    // Fetch the account and check if the fields are set correctly
    const senddit = await program.account.senddit.fetch(authority.publicKey);
    assert.equal(senddit.authority.toBase58(), authority.publicKey.toBase58());
    assert.equal(senddit.treasury.toBase58(), treasury.toBase58());
  });

  it('Initialize the post store', async () => {
    await program.rpc.initPostStore({
      accounts: {
        senddit: authority.publicKey,
        treasury: treasury,
        postStore: authority.publicKey,
        authority: authority.publicKey,
        systemProgram: SystemProgram.programId,
      },
      signers: [authority],
    });

    // Fetch the account and check if the fields are set correctly
    const postStore = await program.account.postStore.fetch(authority.publicKey);
    assert.equal(postStore.posts, 0);
  });

  it('Initialize the comment store', async () => {
    await program.rpc.initCommentStore({
      accounts: {
        senddit: authority.publicKey,
        treasury: treasury,
        commentStore: authority.publicKey,
        authority: authority.publicKey,
        post: authority.publicKey,
        systemProgram: SystemProgram.programId,
      },
      signers: [authority],
    });

    // Fetch the account and check if the fields are set correctly
    const commentStore = await program.account.commentStore.fetch(authority.publicKey);
    assert.equal(commentStore.comments, 0);
  });

  it('Post a link', async () => {
    await program.rpc.postLink(myPost, {
      accounts: {
        senddit: authority.publicKey,
        treasury: treasury,
        postStore: authority.publicKey,
        post: authority.publicKey,
        authority: authority.publicKey,
        posterWallet: authority.publicKey,
        postPda: authority.publicKey,
        systemProgram: SystemProgram.programId,
      },
      signers: [authority],
    });

    // Fetch the account and check if the fields are set correctly
    const post = await program.account.post.fetch(authority.publicKey);
    assert.equal(post.link, myPost);
  });

  it('Upvote a post', async () => { 
    await program.rpc.upvotePost(postUpvote, {
      accounts: {
        senddit: authority.publicKey,
        treasury: treasury,
        postStore: authority.publicKey,
        post: authority.publicKey,
        authority: authority.publicKey,
        posterWallet: authority.publicKey,
        systemProgram: SystemProgram.programId,
      },
      signers: [authority],
    });

    // Fetch the account and check if the fields are set correctly
    const post = await program.account.post.fetch(authority.publicKey);
    assert.equal(post.upvotes, parseInt(postUpvote));
  });

  it('Post a comment', async () => {
    await program.rpc.postComment(myComment, {
      accounts: {
        senddit: authority.publicKey,
        treasury: treasury,
        commentStore: authority.publicKey,
        comment: authority.publicKey,
        authority: authority.publicKey,
        commenterWallet: authority.publicKey,
        post: authority.publicKey,
        systemProgram: SystemProgram.programId,
      },
      signers: [authority],
    });

    // Fetch the account and check if the fields are set correctly
    const comment = await program.account.comment.fetch(authority.publicKey);
    assert.equal(comment.text, myComment);
  });

  it('Upvote a comment', async () => {
    await program.rpc.upvoteComments(commentUpvote, {
      accounts: {
        senddit: authority.publicKey,
        treasury: treasury,
        commentStore: authority.publicKey,
        comment: authority.publicKey,
        authority: authority.publicKey,
        commenterWallet: authority.publicKey,
        post: authority.publicKey,
        systemProgram: anchor.SystemProgram.programId,
      },
      signers: [authority],
    });

    // Fetch the account and check if the fields are set correctly
    const comment = await program.account.comment.fetch(authority.publicKey);
    assert.equal(comment.upvotes, parseInt(commentUpvote));
  });
});
