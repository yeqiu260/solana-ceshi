use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Transfer};

use crate::constants::*;
use crate::error::ErrorCode;
use crate::events::*;
use crate::state::*;
use crate::utils::*;

#[derive(Accounts)]
#[instruction(seed: u64, amount: u64)]
pub struct Withdraw<'info> {
    #[account(
        mut,
        seeds = [STREAM_SEED, payer.key().as_ref(), &seed.to_le_bytes()],
        bump = stream.bump,
        has_one = recipient @ ErrorCode::Unauthorized,
    )]
    pub stream: Account<'info, Stream>,

    #[account(
        mut,
        seeds = [VAULT_SEED, stream.key().as_ref()],
        bump = stream.vault_bump,
    )]
    pub vault: Account<'info, TokenAccount>,

    /// CHECK: PDA authority for the vault
    #[account(
        seeds = [VAULT_SEED, stream.key().as_ref()],
        bump = stream.vault_bump,
    )]
    pub vault_authority: UncheckedAccount<'info>,

    #[account(
        mut,
        token::mint = stream.mint,
    )]
    pub recipient_token_account: Account<'info, TokenAccount>,

    /// CHECK: Payer for PDA derivation only
    pub payer: UncheckedAccount<'info>,

    pub recipient: Signer<'info>,

    pub token_program: Program<'info, Token>,
}

pub fn handler(ctx: Context<Withdraw>, _seed: u64, amount: u64) -> Result<()> {
    let stream = &mut ctx.accounts.stream;
    let clock = Clock::get()?;
    let current_time = clock.unix_timestamp;

    require!(current_time >= stream.start_time, ErrorCode::StreamNotStarted);
    require!(stream.paused_at == 0, ErrorCode::StreamIsPaused);

    let streamed = calculate_streamed_amount(
        stream.rate,
        stream.start_time,
        stream.end_time,
        current_time,
    )?;

    let available = calculate_refund_amount(streamed, stream.withdrawn_amount)?;
    require!(available > 0, ErrorCode::NothingToWithdraw);

    let withdraw_amount = if amount == 0 || amount > available {
        available
    } else {
        amount
    };

    let stream_key = stream.key();
    let seeds = &[VAULT_SEED, stream_key.as_ref(), &[stream.vault_bump]];
    let signer_seeds = &[&seeds[..]];

    let transfer_ctx = CpiContext::new_with_signer(
        token::ID,
        Transfer {
            from: ctx.accounts.vault.to_account_info(),
            to: ctx.accounts.recipient_token_account.to_account_info(),
            authority: ctx.accounts.vault_authority.to_account_info(),
        },
        signer_seeds,
    );
    anchor_spl::token::transfer(transfer_ctx, withdraw_amount)?;

    stream.withdrawn_amount = stream
        .withdrawn_amount
        .checked_add(withdraw_amount)
        .ok_or(ErrorCode::ArithmeticOverflow)?;

    emit!(TokensWithdrawn {
        stream_id: stream_key,
        recipient: stream.recipient,
        amount: withdraw_amount,
        withdrawn_at: current_time,
    });

    Ok(())
}
