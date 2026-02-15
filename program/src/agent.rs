use anchor_lang::prelude::*;
use crate::state::*;
use crate::error::BountyError;

#[derive(Accounts)]
#[instruction(github_handle: String)]
pub struct RegisterAgent<'info> {
    #[account(
        init,
        payer = wallet,
        space = AgentAccount::LEN,
        seeds = [b"agent", github_handle.as_bytes()],
        bump
    )]
    pub agent: Account<'info, AgentAccount>,
    
    #[account(mut)]
    pub wallet: Signer<'info>,
    
    pub system_program: Program<'info, System>,
}

pub fn register_agent(
    ctx: Context<RegisterAgent>,
    github_handle: String,
    signature: [u8; 64],
) -> Result<()> {
    let agent = &mut ctx.accounts.agent;
    let clock = Clock::get()?;
    
    agent.github_handle = github_handle;
    agent.wallet = ctx.accounts.wallet.key();
    agent.signature = signature;
    agent.registered_at = clock.unix_timestamp;
    agent.bump = *ctx.bumps.get("agent").unwrap();
    
    Ok(())
}
