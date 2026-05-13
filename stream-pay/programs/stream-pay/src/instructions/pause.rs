use anchor_lang::prelude::*;

use crate::constants::*;
use crate::error::ErrorCode;
use crate::events::*;
use crate::state::*;

#[derive(Accounts)]
#[instruction(seed: u64)]
pub struct PauseStream<'info> {
    #[account(
        mut,
        seeds = [STREAM_SEED, payer.key().as_ref(), &seed.to_le_bytes()],
        bump = stream.bump,
        has_one = payer @ ErrorCode::Unauthorized,
    )]
    pub stream: Account<'info, Stream>,

    pub payer: Signer<'info>,
}

pub fn handler(ctx: Context<PauseStream>, _seed: u64) -> Result<()> {
    let stream = &mut ctx.accounts.stream;
    let clock = Clock::get()?;
    let current_time = clock.unix_timestamp;

    require!(stream.paused_at == 0, ErrorCode::StreamIsPaused);
    require!(current_time >= stream.start_time, ErrorCode::StreamNotStarted);

    stream.paused_at = current_time;

    emit!(StreamPaused {
        stream_id: stream.key(),
        paused_by: ctx.accounts.payer.key(),
        paused_at: current_time,
    });

    Ok(())
}
