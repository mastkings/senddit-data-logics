use anchor_lang::prelude::*;
use solana_program::{
    program::invoke,
    system_instruction::transfer,
    pubkey::Pubkey,
};
declare_id!("Fb5SwfkQxvcdRyXAc7BKm23irSR5A82vq1Gn1439W9ys");

#[program]
pub mod senddit_data_logics {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        let senddit: &mut Account<Senddit> = &mut ctx.accounts.senddit;
        let authority: &mut Signer = &mut ctx.accounts.authority;

        senddit.authority = authority.key();
        senddit.treasury = authority.key();
        senddit.fee = (0.001 * 1e9) as u64; // 0.001 SOL
        Ok(())
    }

    pub fn init_post_store(ctx: Context<InitPostStore>) -> Result<()> {
        let senddit: &mut Account<Senddit> = &mut ctx.accounts.senddit;
        let treasury: &mut UncheckedAccount = &mut ctx.accounts.treasury;
        let post_store: &mut Account<PostStore> = &mut ctx.accounts.post_store;
        let authority: &mut Signer = &mut ctx.accounts.authority;

        payout_fees(treasury, authority, senddit, None);

        post_store.posts = 0;
        post_store.bump = *ctx.bumps.get("post_store").unwrap();

        Ok(())
    }

    pub fn post_link(ctx: Context<PostLink>, link: String) -> Result<()> {
        let senddit: &mut Account<Senddit> = &mut ctx.accounts.senddit;
        let treasury: &mut UncheckedAccount = &mut ctx.accounts.treasury;
        let post_store: &mut Account<PostStore> = &mut ctx.accounts.post_store;
        let post: &mut Account<Post> = &mut ctx.accounts.post;
        let authority: &mut Signer = &mut ctx.accounts.authority;
        let poster_wallet: &mut UncheckedAccount = &mut ctx.accounts.poster_wallet;

        payout_fees(poster_wallet,authority, senddit, Some(treasury));

        // Make sure string is not empty or too large
        if link.len() == 0 {
            return Err(ErrorCode::NoTextSubmitted.into());
        }
        if link.len() > 196 {
            return Err(ErrorCode::LinkTooLarge.into());
        }

        post_store.posts = post_store
            .posts.checked_add(1)
            .ok_or(ErrorCode::OverflowUnderflow)?;

        post.authority = authority.key();
        post.link = link;
        post.upvotes = 1;
        post.comments = 0;
        post.bump = *ctx.bumps.get("post").unwrap();

        Ok(())
    }

    pub fn upvote_post(ctx: Context<UpvotePost>, _number: String) -> Result<()> {
        let senddit: &mut Account<Senddit> = &mut ctx.accounts.senddit;
        let treasury: &mut UncheckedAccount = &mut ctx.accounts.treasury;
        let post: &mut Account<Post> = &mut ctx.accounts.post;
        let authority: &mut Signer = &mut ctx.accounts.authority;
        let poster_wallet: &mut UncheckedAccount = &mut ctx.accounts.poster_wallet;

        payout_fees(poster_wallet, authority, senddit, Some(treasury));

        post.upvotes = post
            .upvotes.checked_add(1)
            .ok_or(ErrorCode::OverflowUnderflow)?;

        Ok(())
    }

    // Initialize comment store: Reset comment store fields and pay fees
    pub fn init_comment_store(ctx: Context<InitCommentStore>) -> Result<()> {
        // Extract mutable references to accounts
        let senddit: &mut Account<Senddit> = &mut ctx.accounts.senddit;
        let treasury: &mut UncheckedAccount = &mut ctx.accounts.treasury;
        let comment_store: &mut Account<CommentStore> = &mut ctx.accounts.comment_store;
        let authority: &mut Signer = &mut ctx.accounts.authority;

        // Pay fees using payout_fees function (excluding treasury)
        payout_fees(treasury, authority, senddit,  None);

        // Reset comment store fields
        comment_store.comments = 0;
        comment_store.bump = *ctx.bumps.get("comment_store").unwrap();

        Ok(())
    }

    // Post a comment: Store a comment on-chain
    pub fn post_comment(ctx: Context<PostComment>, text: String) -> Result<()> {
        // Extract mutable references to accounts
        let senddit: &mut Account<Senddit> = &mut ctx.accounts.senddit;
        let treasury: &mut UncheckedAccount = &mut ctx.accounts.treasury;
        let comment_store: &mut Account<CommentStore> = &mut ctx.accounts.comment_store;
        let comment: &mut Account<Comment> = &mut ctx.accounts.comment;
        let authority: &mut Signer = &mut ctx.accounts.authority;
        let commenter_wallet: &mut UncheckedAccount = &mut ctx.accounts.commenter_wallet;

        // Pay fees using payout_fees function (including treasury)
        payout_fees(commenter_wallet, authority, senddit, Some(treasury));

        // Check if the comment text is not empty or too large
        if text.len() == 0 {
            return Err(ErrorCode::NoTextSubmitted.into());
        }
        if text.len() > 192 {
            return Err(ErrorCode::CommentTooLarge.into());
        }

        // Increment comment count, handling overflow/underflow
        comment_store.comments = comment_store.comments.checked_add(1).ok_or(ErrorCode::OverflowUnderflow)?;

        // Set comment fields
        comment.authority = authority.key();
        comment.text = text;
        comment.upvotes = 1;
        comment.bump = *ctx.bumps.get("comment").unwrap();

        Ok(())
    }

    // Upvote comments: Increment the upvotes count of a comment
    pub fn upvote_comments(ctx: Context<UpvoteComments>, _number: String) -> Result<()> {
        // Extract mutable references to accounts
        let senddit: &mut Account<Senddit> = &mut ctx.accounts.senddit;
        let treasury: &mut UncheckedAccount = &mut ctx.accounts.treasury;
        let comment: &mut Account<Comment> = &mut ctx.accounts.comment;
        let authority: &mut Signer = &mut ctx.accounts.authority;
        let commenter_wallet: &mut UncheckedAccount = &mut ctx.accounts.commenter_wallet;

        // Pay fees using payout_fees function (including treasury)
        payout_fees(commenter_wallet, authority, senddit, Some(treasury));

        // Increment upvotes count of the comment, handling overflow/underflow
        comment.upvotes = comment.upvotes.checked_add(1).ok_or(ErrorCode::OverflowUnderflow)?;

        Ok(())
    }

}

// Utils

pub fn payout_fees<'info>(
    to: &mut UncheckedAccount<'info>,
    from: &mut Signer<'info>,
    senddit: &mut Account<'info, Senddit>,
    treasury: Option<&mut UncheckedAccount<'info>>
) {
    // first transfer money to the platform
    if let Some(treasury) = treasury {
        invoke(
            &transfer(from.key, treasury.key, senddit.fee),
            &[from.to_account_info(), treasury.to_account_info()],
        )
        .unwrap();
    }
    // then transfer money to the user who posted
    invoke(
        &transfer(from.key, to.key, senddit.fee),
        &[from.to_account_info(), to.to_account_info()],
    )
    .unwrap();
}

// Data Validators

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(init, seeds = [b"senddit".as_ref()], bump, payer = authority, space = Senddit::LEN)]
    pub senddit: Account<'info, Senddit>,
    #[account(mut)]
    pub authority: Signer<'info>,
    pub system_program: Program<'info, System>
}

#[derive(Accounts)]
pub struct InitPostStore<'info> {
    #[account(mut, seeds = [b"senddit".as_ref()], bump = senddit.bump)]
    pub senddit: Account<'info, Senddit>,
    /// CHECK: Account must match our config
    #[account(mut, address = senddit.treasury)]
    pub treasury: UncheckedAccount<'info>,
    #[account(mut)]
    pub authority: Signer<'info>,
    #[account(init, seeds =[(((Clock::get().unwrap().unix_timestamp.abs() as f64) / (60.0 * 60.0 * 24.0)) as u128).to_string().as_bytes().as_ref()], bump, payer = authority, space = PostStore::LEN)]
    pub post_store: Account<'info, PostStore>,
    pub system_program: Program<'info, System>
}

#[derive(Accounts)]
#[instruction(link: String)]
pub struct PostLink<'info> {
    #[account(mut, seeds = [b"senddit".as_ref()], bump = senddit.bump)]
    pub senddit: Account<'info, Senddit>,
    /// CHECK: Account must match our config
    #[account(mut, address = senddit.treasury)]
    pub treasury: UncheckedAccount<'info>,
    #[account(mut)]
    pub authority: Signer<'info>,
    /// CHECK: Account must match the creator of the store
    #[account(mut, address = post_store.authority)]
    pub poster_wallet: UncheckedAccount<'info>,
    #[account(mut,seeds =[(((Clock::get().unwrap().unix_timestamp.abs() as f64) / (60.0 * 60.0 * 24.0)) as u128).to_string().as_bytes().as_ref()], bump = post_store.bump)]
    pub post_store: Account<'info, PostStore>,
    #[account(init, seeds = [post_store.key().as_ref(), (post_store.posts + 1).to_string().as_bytes().as_ref()], bump, payer = authority, space = Post::LEN)]
    pub post: Account<'info, Post>,
    /// CHECK: It's okay not to deserialize this, its just to prevent duplicate links
    #[account(init, seeds = [link.as_bytes().as_ref()], bump, payer = authority, space = 8)]
    pub post_pda: UncheckedAccount<'info>,
    pub system_program: Program<'info, System>
}

#[derive(Accounts)]
#[instruction(number: String)]
pub struct UpvotePost<'info> {
    #[account(mut, seeds = [b"senddit".as_ref()], bump = senddit.bump)]
    pub senddit: Account<'info, Senddit>,
    /// CHECK: Account must match our config
    #[account(mut, address = senddit.treasury)]
    pub treasury: UncheckedAccount<'info>,
    #[account(mut)]
    pub authority: Signer<'info>,
    /// CHECK: Account must match the maker of the post
    #[account(mut, address = post.authority)]
    pub poster_wallet: UncheckedAccount<'info>,
    #[account(mut, seeds = [post.key().as_ref()], bump = post_store.bump)]
    pub post_store: Account<'info, PostStore>,
    #[account(mut, seeds = [post_store.key().as_ref(), number.as_bytes().as_ref()], bump = post.bump)]
    pub post: Account<'info, Post>,
    pub system_program: Program<'info, System>
}

#[derive(Accounts)]
pub struct InitCommentStore<'info> {
    #[account(mut, seeds = [b"senddit".as_ref()], bump = senddit.bump)]
    pub senddit: Account<'info, Senddit>,
    /// CHECK: Account must match our config
    #[account(mut, address = senddit.treasury)]
    pub treasury: UncheckedAccount<'info>,
    #[account(mut)]
    pub authority: Signer<'info>,
    #[account(init, seeds = [post.key().as_ref()], bump, payer = authority, space = CommentStore::LEN)]
    pub comment_store: Account<'info, CommentStore>,
    pub post: Account<'info, Post>,
    pub system_program: Program<'info, System>
}

#[derive(Accounts)]
pub struct PostComment<'info> {
    #[account(mut, seeds = [b"senddit".as_ref()], bump = senddit.bump)]
    pub senddit: Account<'info, Senddit>,
    /// CHECK: Account must match our config
    #[account(mut, address = senddit.treasury)]
    pub treasury: UncheckedAccount<'info>,
    #[account(mut)]
    pub authority: Signer<'info>,
    /// CHECK: Account must match the creator of the store
    #[account(mut, address = comment_store.authority)]
    pub commenter_wallet: UncheckedAccount<'info>,
    #[account(mut, seeds = [post.key().as_ref()], bump = comment_store.bump)]
    pub comment_store: Account<'info, CommentStore>,
    #[account(mut)]
    pub post: Account<'info, Post>,
    #[account(init, seeds = [comment_store.key().as_ref(),(comment_store.comments + 1).to_string().as_bytes().as_ref()], bump, payer = authority, space = Comment::LEN)]
    pub comment: Account<'info, Comment>,
    pub system_program: Program<'info, System>
}

#[derive(Accounts)]
#[instruction(number: String)]
pub struct UpvoteComments<'info> {
    #[account(mut, seeds = [b"senddit".as_ref()], bump = senddit.bump)]
    pub senddit: Account<'info, Senddit>,
    /// CHECK: Account must match our config
    #[account(mut, address = senddit.treasury)]
    pub treasury: UncheckedAccount<'info>,
    #[account(mut)]
    pub authority:  Signer<'info>,
    /// CHECK: Account must match the maker of the comments
    #[account(mut, address = comment.authority)]
    pub commenter_wallet: UncheckedAccount<'info>,
    #[account(mut, seeds = [post.key().as_ref()], bump = comment_store.bump)]
    pub comment_store: Account<'info, CommentStore>,
    #[account(mut)]
    pub post: Account<'info, Post>,
    #[account(mut, seeds = [comment_store.key().as_ref(), number.as_bytes().as_ref()], bump = comment.bump)]
    pub comment: Account<'info, Comment>,
    pub system_program: Program<'info, System>
}

// Data structures

const DISCRIMINATOR: usize = 8;
const PUBKEY: usize = 32;
const UNSIGNED_64: usize = 8;
const UNSIGNED_128: usize = 16;
const STRING_PREFIX: usize = 4;
const MAX_LINK_SIZE: usize = 96 * 4;
const MAX_COMMENT_SIZE: usize = 192 * 4;
const BUMP: usize = 1;

#[account]
pub struct Senddit {
    pub authority: Pubkey,
    pub treasury: Pubkey,
    pub fee: u64,
    pub bump: u8
}

impl Senddit {
    pub const LEN: usize = DISCRIMINATOR + PUBKEY + PUBKEY + UNSIGNED_64 + BUMP;
}

#[account]
pub struct PostStore {
    pub authority: Pubkey,
    pub posts: u128,
    pub bump: u8
}

impl PostStore {
    pub const LEN: usize = DISCRIMINATOR + PUBKEY + UNSIGNED_128 + BUMP;
}

#[account]
pub struct Post {
    pub authority: Pubkey,
    pub link: String,
    pub upvotes: u64,
    pub comments: u64,
    pub bump: u8
}

impl Post {
    pub const LEN: usize = DISCRIMINATOR + PUBKEY + STRING_PREFIX + MAX_LINK_SIZE + UNSIGNED_64 + UNSIGNED_64 + BUMP;
}

#[account]
pub struct CommentStore {
    pub authority: Pubkey,
    pub comments: u128,
    pub bump: u8
}

impl CommentStore {
    pub const LEN: usize = DISCRIMINATOR + PUBKEY + UNSIGNED_128 + BUMP;
}

#[account]
pub struct Comment {
    pub authority: Pubkey,
    pub text: String,
    pub upvotes: u64,
    pub bump: u8
}

impl Comment {
    pub const LEN: usize = DISCRIMINATOR + PUBKEY + STRING_PREFIX + MAX_COMMENT_SIZE + UNSIGNED_64 + BUMP;
}

// Error Codes

#[error_code]
pub enum ErrorCode {
    LinkAlreadySubmitted,
    OverflowUnderflow,
    NoTextSubmitted,
    LinkTooLarge,
    CommentTooLarge
}
