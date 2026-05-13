use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Transfer, CloseAccount};

use crate::constants::*;
use crate::error::ErrorCode;
use crate::events::*;
use crate::state::*;
use crate::utils::*;

#[derive(Accounts)]
#[instruction(seed: u64)]
pub struct CloseStream<'info> {
    #[account(
        mut,
        seeds = [STREAM_SEED, payer.key().as_ref(), &seed.to_le_bytes()],
        bump = stream.bump,
        has_one = payer @ ErrorCode::Unauthorized,
        close = payer,
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
    pub payer_token_account: Account<'info, TokenAccount>,

    #[account(
        mut,
        token::mint = stream.mint,
    )]
    pub recipient_token_account: Account<'info, TokenAccount>,

    #[account(mut)]
    pub payer: Signer<'info>,

    pub token_program: Program<'info, Token>,
}

pub fn handler(ctx: Context<CloseStream>, _seed: u64) -> Result<()> {
    let stream = &ctx.accounts.stream;
    let clock = Clock::get()?;
    let current_time = clock.unix_timestamp;

    let effective_end = if stream.paused_at != 0 {
        stream.paused_at
    } else {
        current_time
    };

    let streamed = calculate_streamed_amount(
        stream.rate,
        stream.start_time,
        stream.end_time,
        effective_end,
    )?;

    let pending = calculate_refund_amount(streamed, stream.withdrawn_amount)?;
    let refund = calculate_refund_amount(stream.total_amount, streamed)?;

    let stream_key = stream.key();
    let seeds = &[VAULT_SEED, stream_key.as_ref(), &[stream.vault_bump]];
    let signer_seeds = &[&seeds[..]];

    // Transfer pending streamed amount to recipient
    if pending > 0 {
        let transfer_ctx = CpiContext::new_with_signer(
            token::ID,
            Transfer {
                from: ctx.accounts.vault.to_account_info(),
                to: ctx.accounts.recipient_token_account.to_account_info(),
                authority: ctx.accounts.vault_authority.to_account_info(),
            },
            signer_seeds,
        );
        anchor_spl::token::transfer(transfer_ctx, pending)?;
    }

    // Transfer remaining balance back to payer
    let remaining = ctx.accounts.vault.amount;
    if remaining > 0 {
        let transfer_ctx = CpiContext::new_with_signer(
            token::ID,
            Transfer {
                from: ctx.accounts.vault.to_account_info(),
                to: ctx.accounts.payer_token_account.to_account_info(),
                authority: ctx.accounts.vault_authority.to_account_info(),
            },
            signer_seeds,
        );
        anchor_spl::token::transfer(transfer_ctx, remaining)?;
    }

    // Close vault - return rent to payer
    let close_ctx = CpiContext::new_with_signer(
        token::ID,
        CloseAccount {
            account: ctx.accounts.vault.to_account_info(),
            destination: ctx.accounts.payer.to_account_info(),
            authority: ctx.accounts.vault_authority.to_account_info(),
        },
        signer_seeds,
    );
    anchor_spl::token::close_account(close_ctx)?;

    emit!(StreamClosed {
        stream_id: stream_key,
        closed_by: ctx.accounts.payer.key(),
        refunded_amount: refund,
        closed_at: current_time,
    });

    Ok(())
}
