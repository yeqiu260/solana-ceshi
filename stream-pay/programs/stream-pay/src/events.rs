use anchor_lang::prelude::*;

#[event]
pub struct StreamCreated {
    pub stream_id: Pubkey,
    pub payer: Pubkey,
    pub recipient: Pubkey,
    pub mint: Pubkey,
    pub rate: u64,
    pub total_amount: u64,
    pub start_time: i64,
    pub end_time: i64,
}

#[event]
pub struct TokensWithdrawn {
    pub stream_id: Pubkey,
    pub recipient: Pubkey,
    pub amount: u64,
    pub withdrawn_at: i64,
}

#[event]
pub struct StreamPaused {
    pub stream_id: Pubkey,
    pub paused_by: Pubkey,
    pub paused_at: i64,
}

#[event]
pub struct StreamResumed {
    pub stream_id: Pubkey,
    pub resumed_by: Pubkey,
    pub resumed_at: i64,
}

#[event]
pub struct StreamClosed {
    pub stream_id: Pubkey,
    pub closed_by: Pubkey,
    pub refunded_amount: u64,
    pub closed_at: i64,
}

#[event]
pub struct RateAdjusted {
    pub stream_id: Pubkey,
    pub old_rate: u64,
    pub new_rate: u64,
    pub adjusted_by: Pubkey,
}
