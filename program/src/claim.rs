use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Transfer};
use crate::state::*;
use crate::error::BountyError;
use crate::FEE_BPS;

#[derive(Accounts)]
#[instruction(github_handle: String)]
pub struct ClaimBounty<'info> {
    #[account(
        constraint = bounty.status == BountyStatus::Funded @ BountyError::BountyNotFunded
    )]
    pub bounty: Account<'info, BountyAccount>,
    
    #[account(
        init,
        payer = claimant,
        space = ClaimAccount::LEN,
        seeds = [b"claim", bounty.key().as_ref(), github_handle.as_bytes()],
        bump
    )]
    pub claim: Account<'info, ClaimAccount>,
    
    #[account(
        seeds = [b"agent", github_handle.as_bytes()],
        bump,
        constraint = agent.wallet == claimant.key() @ BountyError::AgentNotRegistered
    )]
    pub agent: Account<'info, AgentAccount>,
    
    #[account(mut)]
    pub claimant: Signer<'info>,
    
    pub system_program: Program<'info, System>,
}

pub fn claim_bounty(
    ctx: Context<ClaimBounty>,
    github_handle: String,
) -> Result<()> {
    let claim = &mut ctx.accounts.claim;
    
    claim.bounty = ctx.accounts.bounty.key();
    claim.claimant = ctx.accounts.claimant.key();
    claim.github_handle = github_handle;
    claim.commits = 0;
    claim.share_bps = 0;
    claim.withdrawn = false;
    claim.bump = *ctx.bumps.get("claim").unwrap();
    
    Ok(())
}

#[derive(Accounts)]
#[instruction(contributors: Vec<ContributorInfo>)]
pub struct ReleaseOnMerge<'info> {
    #[account(
        mut,
        constraint = bounty.status == BountyStatus::Funded @ BountyError::BountyNotFunded
    )]
    pub bounty: Account<'info, BountyAccount>,
    
    #[account(
        mut,
        seeds = [b"escrow", bounty.key().as_ref()],
        bump
    )]
    pub escrow: Account<'info, TokenAccount>,
    
    #[account(
        mut,
        seeds = [b"treasury"],
        bump
    )]
    pub treasury: Account<'info, TreasuryAccount>,
    
    #[account(
        mut,
        token::mint = bounty.token_mint,
        token::authority = treasury
    )]
    pub treasury_token: Account<'info, TokenAccount>,
    
    /// Bot authority - must be whitelisted
    #[account(constraint = authority.key() == treasury.authority @ BountyError::NotAuthorized)]
    pub authority: Signer<'info>,
    
    pub token_program: Program<'info, Token>,
}

pub fn release_on_merge(
    ctx: Context<ReleaseOnMerge>,
    contributors: Vec<ContributorInfo>,
) -> Result<()> {
    let bounty = &mut ctx.accounts.bounty;
    
    let is_native_sol = bounty.token_mint == solana_program::native_token::id();
    
    let (fee, payout) = if is_native_sol {
        (0u64, bounty.amount)
    } else {
        let fee = bounty.amount * FEE_BPS / 10000;
        (fee, bounty.amount - fee)
    };
    
    if fee > 0 {
        let seeds = &[
            b"bounty",
            bounty.repo_id.to_le_bytes().as_ref(),
            bounty.issue_number.to_le_bytes().as_ref(),
            &[bounty.bump],
        ];
        let signer = &[&seeds[..]];
        
        let cpi_ctx = CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            Transfer {
                from: ctx.accounts.escrow.to_account_info(),
                to: ctx.accounts.treasury_token.to_account_info(),
                authority: ctx.accounts.bounty.to_account_info(),
            },
            signer,
        );
        
        token::transfer(cpi_ctx, fee)?;
    }
    
    bounty.status = BountyStatus::Released;
    
    Ok(())
}

#[derive(Accounts)]
pub struct Withdraw<'info> {
    #[account(
        mut,
        constraint = claim.withdrawn == false @ BountyError::AlreadyWithdrawn
    )]
    pub claim: Account<'info, ClaimAccount>,
    
    #[account(
        mut,
        constraint = bounty.status == BountyStatus::Released @ BountyError::BountyNotReleased
    )]
    pub bounty: Account<'info, BountyAccount>,
    
    #[account(
        mut,
        seeds = [b"escrow", bounty.key().as_ref()],
        bump
    )]
    pub escrow: Account<'info, TokenAccount>,
    
    #[account(
        mut,
        constraint = claimant.key() == claim.claimant @ BountyError::NotBountyCreator
    )]
    pub claimant: Signer<'info>,
    
    #[account(mut)]
    pub claimant_token: Account<'info, TokenAccount>,
    
    pub token_program: Program<'info, Token>,
}

pub fn withdraw(ctx: Context<Withdraw>) -> Result<()> {
    let claim = &mut ctx.accounts.claim;
    
    require!(claim.share_bps > 0, BountyError::NoCommitsAllocated);
    
    let is_native_sol = ctx.accounts.bounty.token_mint == solana_program::native_token::id();
    let (_, payout) = if is_native_sol {
        (0u64, ctx.accounts.bounty.amount)
    } else {
        let fee = ctx.accounts.bounty.amount * FEE_BPS / 10000;
        (fee, ctx.accounts.bounty.amount - fee)
    };
    
    let amount = payout * claim.share_bps as u64 / 10000;
    
    if amount > 0 {
        let seeds = &[
            b"bounty",
            ctx.accounts.bounty.repo_id.to_le_bytes().as_ref(),
            ctx.accounts.bounty.issue_number.to_le_bytes().as_ref(),
            &[ctx.accounts.bounty.bump],
        ];
        let signer = &[&seeds[..]];
        
        let cpi_ctx = CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            Transfer {
                from: ctx.accounts.escrow.to_account_info(),
                to: ctx.accounts.claimant_token.to_account_info(),
                authority: ctx.accounts.bounty.to_account_info(),
            },
            signer,
        );
        
        token::transfer(cpi_ctx, amount)?;
    }
    
    claim.withdrawn = true;
    
    Ok(())
}
