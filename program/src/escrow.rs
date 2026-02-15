use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Transfer};
use crate::state::*;
use crate::error::BountyError;

#[derive(Accounts)]
pub struct Deposit<'info> {
    #[account(
        mut,
        has_one = creator @ BountyError::NotBountyCreator
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

pub fn deposit(ctx: Context<Deposit>, amount: u64) -> Result<()> {
    require!(amount > 0, BountyError::InvalidAmount);
    
    let cpi_ctx = CpiContext::new(
        ctx.accounts.token_program.to_account_info(),
        Transfer {
            from: ctx.accounts.creator_token.to_account_info(),
            to: ctx.accounts.escrow.to_account_info(),
            authority: ctx.accounts.creator.to_account_info(),
        },
    );
    
    token::transfer(cpi_ctx, amount)?;
    
    let bounty = &mut ctx.accounts.bounty;
    bounty.amount = bounty.amount.checked_add(amount).unwrap();
    bounty.status = BountyStatus::Funded;
    
    Ok(())
}

#[derive(Accounts)]
pub struct InitializeTreasury<'info> {
    #[account(
        init,
        payer = authority,
        space = TreasuryAccount::LEN,
        seeds = [b"treasury"],
        bump
    )]
    pub treasury: Account<'info, TreasuryAccount>,
    
    #[account(mut)]
    pub authority: Signer<'info>,
    
    pub system_program: Program<'info, System>,
}

pub fn initialize_treasury(ctx: Context<InitializeTreasury>, fee_bps: u64) -> Result<()> {
    let treasury = &mut ctx.accounts.treasury;
    
    treasury.authority = ctx.accounts.authority.key();
    treasury.fee_bps = fee_bps;
    treasury.bump = *ctx.bumps.get("treasury").unwrap();
    
    Ok(())
}
