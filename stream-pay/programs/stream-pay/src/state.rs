use anchor_lang::prelude::*;

/// Stream account holding the state of a pay-per-second token stream.
#[account]
pub struct Stream {
    /// The payer who deposits and funds the stream
    pub payer: Pubkey,
    /// The recipient who receives the streamed tokens
    pub recipient: Pubkey,
    /// The SPL token mint
    pub mint: Pubkey,
    /// The vault token account (PDA) holding deposited tokens
    pub vault: Pubkey,
    /// Token streaming rate per second, scaled by PRECISION (1e9)
    pub rate: u64,
    /// Total amount of tokens deposited
    pub total_amount: u64,
    /// Total amount already withdrawn by recipient
    pub withdrawn_amount: u64,
    /// Unix timestamp when streaming starts
    pub start_time: i64,
    /// Unix timestamp when streaming ends (0 = open-ended)
    pub end_time: i64,
    /// Timestamp when stream was paused (0 = not paused)
    pub paused_at: i64,
    /// User-provided seed for PDA derivation (allows multiple streams per pair)
    pub seed: u64,
    /// PDA bump for the vault token account
    pub vault_bump: u8,
    /// PDA bump for the stream account
    pub bump: u8,
}

impl Stream {
    pub const LEN: usize = 8  // discriminator
        + 32  // payer
        + 32  // recipient
        + 32  // mint
        + 32  // vault
        + 8   // rate
        + 8   // total_amount
        + 8   // withdrawn_amount
        + 8   // start_time
        + 8   // end_time
        + 8   // paused_at
        + 8   // seed
        + 1   // vault_bump
        + 1;  // bump
}
