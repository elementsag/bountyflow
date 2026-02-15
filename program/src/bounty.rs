use anchor_lang::prelude::*;
use anchor_spl::token::{Token, TokenAccount};
use crate::state::*;
use crate::error::BountyError;

#[derive(Accounts)]
#[instruction(repo_id: u64, issue_number: u64, amount: u64, token_mint: Pubkey)]
pub struct CreateBounty<'info> {
    #[account(
        init,
        payer = creator,
        space = BountyAccount::LEN,
        seeds = [b"bounty", repo_id.to_le_bytes().as_ref(), issue_number.to_le_bytes().as_ref()],
        bump
    )]
    pub bounty: Account<'info, BountyAccount>,
    
    #[account(
        init_if_needed,
        payer = creator,
        token::mint = token_mint,
        token::authority = bounty,
        seeds = [b"escrow", bounty.key().as_ref()],
        bump
    )]
    pub escrow: Account<'info, TokenAccount>,
    
    #[account(mut)]
    pub creator: Signer<'info>,
    
    pub token_mint: AccountInfo<'info>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

pub fn create_bounty(
    ctx: Context<CreateBounty>,
    repo_id: u64,
    issue_number: u64,
    amount: u64,
    token_mint: Pubkey,
) -> Result<()> {
    require!(amount > 0, BountyError::InvalidAmount);
    
    let bounty = &mut ctx.accounts.bounty;
    let clock = Clock::get()?;
    
    bounty.repo_id = repo_id;
    bounty.issue_number = issue_number;
    bounty.creator = ctx.accounts.creator.key();
    bounty.amount = amount;
    bounty.token_mint = token_mint;
    bounty.status = BountyStatus::Open;
    bounty.created_at = clock.unix_timestamp;
    bounty.bump = *ctx.bumps.get("bounty").unwrap();
    
    Ok(())
}

#[derive(Accounts)]
pub struct CancelBounty<'info> {
    #[account(
        mut,
        has_one = creator @ BountyError::NotBountyCreator,
        constraint = bounty.status == BountyStatus::Funded || bounty.status == BountyStatus::Open @ BountyError::BountyNotFunded
    )]
    pub bounty: Account<'info, BountyAccount>,
    
    #[account(
        mut,
        seeds = [b"escrow", bounty.key().as_ref()],
        bump
    )]
    pub escrow: Account<'info, TokenAccount>,
    
    #[account(mut)]
    pub creator: Signer<'info>,
    
    #[account(mut)]
    pub creator_token_account: Account<'info, TokenAccount>,
    
    pub token_program: Program<'info, Token>,
}

pub fn cancel_bounty(ctx: Context<CancelBounty>) -> Result<()> {
    let bounty = &mut ctx.accounts.bounty;
    bounty.status = BountyStatus::Closed;
    
    if bounty.amount > 0 {
        let seeds = &[
            b"bounty",
            bounty.repo_id.to_le_bytes().as_ref(),
            bounty.issue_number.to_le_bytes().as_ref(),
            &[bounty.bump],
        ];
        let signer = &[&seeds[..]];
        
        let cpi_ctx = CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            anchor_spl::token::Transfer {
                from: ctx.accounts.escrow.to_account_info(),
                to: ctx.accounts.creator_token_account.to_account_info(),
                authority: ctx.accounts.bounty.to_account_info(),
            },
            signer,
        );
        
        anchor_spl::token::transfer(cpi_ctx, bounty.amount)?;
    }
    
    Ok(())
}

#[derive(Accounts)]
pub struct RefundTimeout<'info> {
    #[account(
        mut,
        constraint = bounty.status == BountyStatus::Funded @ BountyError::BountyNotFunded
    )]
    pub bounty: Account<'info, BountyAccount>,
    
    #[account(mut)]
    pub creator: Signer<'info>,
    
    pub system_program: Program<'info, System>,
}

pub fn refund_timeout(ctx: Context<RefundTimeout>) -> Result<()> {
    let bounty = &ctx.accounts.bounty;
    let clock = Clock::get()?;
    
    require!(
        clock.unix_timestamp >= bounty.created_at + crate::REFUND_TIMEOUT_SECONDS,
        BountyError::TimeoutNotReached
    );
    
    Ok(())
}
