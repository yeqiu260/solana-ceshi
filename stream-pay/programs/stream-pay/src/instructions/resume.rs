use anchor_lang::prelude::*;

use crate::constants::*;
use crate::error::ErrorCode;
use crate::events::*;
use crate::state::*;

#[derive(Accounts)]
#[instruction(seed: u64)]
pub struct ResumeStream<'info> {
    #[account(
        mut,
        seeds = [STREAM_SEED, payer.key().as_ref(), &seed.to_le_bytes()],
        bump = stream.bump,
        has_one = payer @ ErrorCode::Unauthorized,
    )]
    pub stream: Account<'info, Stream>,

    pub payer: Signer<'info>,
}

pub fn handler(ctx: Context<ResumeStream>, _seed: u64) -> Result<()> {
    let stream = &mut ctx.accounts.stream;
    let clock = Clock::get()?;
    let current_time = clock.unix_timestamp;

    require!(stream.paused_at != 0, ErrorCode::StreamNotPaused);

    let pause_duration = current_time
        .checked_sub(stream.paused_at)
        .ok_or(ErrorCode::ArithmeticOverflow)?;

    stream.start_time = stream
        .start_time
        .checked_add(pause_duration)
        .ok_or(ErrorCode::ArithmeticOverflow)?;
    stream.paused_at = 0;

    emit!(StreamResumed {
        stream_id: stream.key(),
        resumed_by: ctx.accounts.payer.key(),
        resumed_at: current_time,
    });

    Ok(())
}
