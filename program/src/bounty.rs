use anchor_lang::prelude::*;
use anchor_spl::token::{Token, TokenAccount};
use crate::state::*;
use crate::error::BountyError;

#[derive(Accounts)]
#[instruction(repo_id: u64, issue_number: u64)]
pub struct CreateBounty<'info> {
    #[account(
        init,
        payer = creator,
        space = BountyAccount::LEN,
        seeds = [b"bounty", repo_id.to_le_bytes().as_ref(), issue_number.to_le_bytes().as_ref()],
        bump
    )]
    pub bounty: Account<'info, BountyAccount>,
    
    #[account(mut)]
    pub creator: Signer<'info>,
    
    pub system_program: Program<'info, System>,
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
    bounty.amount = 0; // Starts at 0, funded via deposit
    bounty.token_mint = token_mint;
    bounty.status = BountyStatus::Open;
    bounty.created_at = clock.unix_timestamp;
    bounty.bump = *ctx.bumps.get("bounty").unwrap();
    
    Ok(())
}

#[derive(Accounts)]
pub struct Deposit<'info> {
    #[account(
        mut,
        has_one = creator @ BountyError::NotBountyCreator,
        has_one = token_mint
    )]
    pub bounty: Account<'info, BountyAccount>,
    
    #[account(
        mut,
        seeds = [b"escrow", bounty.key().as_ref()],
        bump,
        token::mint = token_mint,
        token::authority = bounty
    )]
    pub escrow: Account<'info, TokenAccount>,
    
    #[account(mut)]
    pub creator: Signer<'info>,
    
    #[account(mut)]
    pub creator_token: Account<'info, TokenAccount>,
    
    pub token_mint: AccountInfo<'info>,
    pub token_program: Program<'info, Token>,
}

pub fn deposit(ctx: Context<Deposit>, amount: u64) -> Result<()> {
    require!(amount > 0, BountyError::InvalidAmount);
    
    anchor_spl::token::transfer(
        CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            anchor_spl::token::Transfer {
                from: ctx.accounts.creator_token.to_account_info(),
                to: ctx.accounts.escrow.to_account_info(),
                authority: ctx.accounts.creator.to_account_info(),
            },
        ),
        amount,
    )?;
    
    let bounty = &mut ctx.accounts.bounty;
    bounty.amount = bounty.amount.checked_add(amount).unwrap();
    bounty.status = BountyStatus::Funded;
    
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
    pub creator_token: Account<'info, TokenAccount>,
    
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
        
        anchor_spl::token::transfer(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                anchor_spl::token::Transfer {
                    from: ctx.accounts.escrow.to_account_info(),
                    to: ctx.accounts.creator_token.to_account_info(),
                    authority: ctx.accounts.bounty.to_account_info(),
                },
                &[&seeds[..]],
            ),
            bounty.amount,
        )?;
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
