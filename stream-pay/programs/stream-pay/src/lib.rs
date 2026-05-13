pub mod constants;
pub mod error;
pub mod events;
pub mod instructions;
pub mod state;
pub mod tests;
pub mod utils;

use anchor_lang::prelude::*;

pub use constants::*;
pub use events::*;
pub use state::*;

// Re-export account structs for the #[program] macro
pub use instructions::create_stream::CreateStream;
pub use instructions::withdraw::Withdraw;
pub use instructions::pause::PauseStream;
pub use instructions::resume::ResumeStream;
pub use instructions::close::CloseStream;
pub use instructions::adjust_rate::AdjustRate;

// Re-export generated client account types (pub(crate) to match derive visibility)
pub(crate) use instructions::create_stream::__client_accounts_create_stream;
pub(crate) use instructions::withdraw::__client_accounts_withdraw;
pub(crate) use instructions::pause::__client_accounts_pause_stream;
pub(crate) use instructions::resume::__client_accounts_resume_stream;
pub(crate) use instructions::close::__client_accounts_close_stream;
pub(crate) use instructions::adjust_rate::__client_accounts_adjust_rate;

declare_id!("Etsz3vqLMqfPToiVB1ECb2aCM1swRSaa9XLi6boH38uL");

#[program]
pub mod stream_pay {
    use super::*;

    pub fn create_stream(
        ctx: Context<CreateStream>,
        seed: u64,
        amount: u64,
        rate: u64,
        start_time: i64,
        end_time: i64,
    ) -> Result<()> {
        instructions::create_stream::handler(ctx, seed, amount, rate, start_time, end_time)
    }

    pub fn withdraw(
        ctx: Context<Withdraw>,
        seed: u64,
        amount: u64,
    ) -> Result<()> {
        instructions::withdraw::handler(ctx, seed, amount)
    }

    pub fn pause_stream(
        ctx: Context<PauseStream>,
        seed: u64,
    ) -> Result<()> {
        instructions::pause::handler(ctx, seed)
    }

    pub fn resume_stream(
        ctx: Context<ResumeStream>,
        seed: u64,
    ) -> Result<()> {
        instructions::resume::handler(ctx, seed)
    }

    pub fn close_stream(
        ctx: Context<CloseStream>,
        seed: u64,
    ) -> Result<()> {
        instructions::close::handler(ctx, seed)
    }

    pub fn adjust_rate(
        ctx: Context<AdjustRate>,
        seed: u64,
        new_rate: u64,
    ) -> Result<()> {
        instructions::adjust_rate::handler(ctx, seed, new_rate)
    }
}
