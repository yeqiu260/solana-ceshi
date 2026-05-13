use anchor_lang::prelude::*;
use anchor_spl::token::{self, Mint, Token, TokenAccount, Transfer};

use crate::constants::*;
use crate::error::ErrorCode;
use crate::events::*;
use crate::state::*;
use crate::utils::*;

#[derive(Accounts)]
#[instruction(seed: u64, amount: u64, rate: u64, start_time: i64, end_time: i64)]
pub struct CreateStream<'info> {
    #[account(
        init,
        payer = payer,
        space = Stream::LEN,
        seeds = [STREAM_SEED, payer.key().as_ref(), &seed.to_le_bytes()],
        bump
    )]
    pub stream: Account<'info, Stream>,

    #[account(
        init,
        payer = payer,
        token::mint = mint,
        token::authority = vault_authority,
        seeds = [VAULT_SEED, stream.key().as_ref()],
        bump
    )]
    pub vault: Account<'info, TokenAccount>,

    /// CHECK: PDA authority for the vault
    #[account(
        seeds = [VAULT_SEED, stream.key().as_ref()],
        bump
    )]
    pub vault_authority: UncheckedAccount<'info>,

    #[account(
        mut,
        token::mint = mint,
        token::authority = payer,
    )]
    pub payer_token_account: Account<'info, TokenAccount>,

    /// CHECK: Recipient pubkey, validated by being stored in the stream
    pub recipient: UncheckedAccount<'info>,

    #[account(mut)]
    pub payer: Signer<'info>,

    pub mint: Account<'info, Mint>,

    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

pub fn handler(
    ctx: Context<CreateStream>,
    seed: u64,
    amount: u64,
    rate: u64,
    start_time: i64,
    end_time: i64,
) -> Result<()> {
    let stream = &mut ctx.accounts.stream;
    let clock = Clock::get()?;
    let current_time = clock.unix_timestamp;

    require!(amount > 0, ErrorCode::InvalidAmount);
    validate_rate(rate)?;
    require!(start_time > current_time, ErrorCode::InvalidTimestamp);
    if end_time != 0 {
        validate_duration(start_time, end_time)?;
    }

    let transfer_ctx = CpiContext::new(
        token::ID,
        Transfer {
            from: ctx.accounts.payer_token_account.to_account_info(),
            to: ctx.accounts.vault.to_account_info(),
            authority: ctx.accounts.payer.to_account_info(),
        },
    );
    anchor_spl::token::transfer(transfer_ctx, amount)?;

    stream.payer = ctx.accounts.payer.key();
    stream.recipient = ctx.accounts.recipient.key();
    stream.mint = ctx.accounts.mint.key();
    stream.vault = ctx.accounts.vault.key();
    stream.rate = rate;
    stream.total_amount = amount;
    stream.withdrawn_amount = 0;
    stream.start_time = start_time;
    stream.end_time = end_time;
    stream.paused_at = 0;
    stream.seed = seed;
    stream.vault_bump = ctx.bumps.vault;
    stream.bump = ctx.bumps.stream;

    emit!(StreamCreated {
        stream_id: stream.key(),
        payer: stream.payer,
        recipient: stream.recipient,
        mint: stream.mint,
        rate,
        total_amount: amount,
        start_time,
        end_time,
    });

    Ok(())
}
