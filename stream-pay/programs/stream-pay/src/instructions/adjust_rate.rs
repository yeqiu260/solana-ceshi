use anchor_lang::prelude::*;

use crate::constants::*;
use crate::error::ErrorCode;
use crate::events::*;
use crate::state::*;
use crate::utils::*;

#[derive(Accounts)]
#[instruction(seed: u64, new_rate: u64)]
pub struct AdjustRate<'info> {
    #[account(
        mut,
        seeds = [STREAM_SEED, payer.key().as_ref(), &seed.to_le_bytes()],
        bump = stream.bump,
        has_one = payer @ ErrorCode::Unauthorized,
    )]
    pub stream: Account<'info, Stream>,

    pub payer: Signer<'info>,
}

pub fn handler(ctx: Context<AdjustRate>, _seed: u64, new_rate: u64) -> Result<()> {
    let stream = &mut ctx.accounts.stream;

    validate_rate(new_rate)?;

    let old_rate = stream.rate;
    stream.rate = new_rate;

    emit!(RateAdjusted {
        stream_id: stream.key(),
        old_rate,
        new_rate,
        adjusted_by: ctx.accounts.payer.key(),
    });

    Ok(())
}
