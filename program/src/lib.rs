use anchor_lang::prelude::*;

declare_id!("FDaH7vK76MjpAUxhLF4YgpSy5NaPwio29W5kV8HZGwsk");

pub mod state;
pub mod error;
pub mod bounty;
pub mod claim;
pub mod agent;
pub mod escrow;

use bounty::*;
use claim::*;
use agent::*;
use escrow::*;

pub const FEE_BPS: u64 = 250; // 2.5%
pub const REFUND_TIMEOUT_SECONDS: i64 = 30 * 24 * 60 * 60; // 30 days

#[program]
pub mod bountyflow {
    use super::*;

    pub fn create_bounty(
        ctx: Context<CreateBounty>,
        repo_id: u64,
        issue_number: u64,
        amount: u64,
        token_mint: Pubkey,
    ) -> Result<()> {
        bounty::create_bounty(ctx, repo_id, issue_number, amount, token_mint)
    }

    pub fn init_escrow(ctx: Context<InitEscrow>) -> Result<()> {
        escrow::init_escrow(ctx)
    }

    pub fn deposit(ctx: Context<Deposit>, amount: u64) -> Result<()> {
        bounty::deposit(ctx, amount)
    }

    pub fn claim_bounty(
        ctx: Context<ClaimBounty>,
        github_handle: String,
    ) -> Result<()> {
        claim::claim_bounty(ctx, github_handle)
    }

    pub fn register_agent(
        ctx: Context<RegisterAgent>,
        github_handle: String,
        signature: [u8; 64],
    ) -> Result<()> {
        agent::register_agent(ctx, github_handle, signature)
    }

    pub fn release_on_merge(
        ctx: Context<ReleaseOnMerge>,
        contributors: Vec<ContributorInfo>,
    ) -> Result<()> {
        claim::release_on_merge(ctx, contributors)
    }

    pub fn withdraw(ctx: Context<Withdraw>) -> Result<()> {
        claim::withdraw(ctx)
    }

    pub fn cancel_bounty(ctx: Context<CancelBounty>) -> Result<()> {
        bounty::cancel_bounty(ctx)
    }

    pub fn refund_timeout(ctx: Context<RefundTimeout>) -> Result<()> {
        bounty::refund_timeout(ctx)
    }

    pub fn initialize_treasury(ctx: Context<InitializeTreasury>, fee_bps: u64) -> Result<()> {
        escrow::initialize_treasury(ctx, fee_bps)
    }
}
