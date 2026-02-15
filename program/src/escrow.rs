use anchor_lang::prelude::*;
use anchor_spl::token::{Token, TokenAccount};
use crate::state::*;
use crate::error::BountyError;

#[derive(Accounts)]
pub struct InitEscrow<'info> {
    #[account(has_one = creator @ BountyError::NotBountyCreator)]
    pub bounty: Account<'info, BountyAccount>,
    
    #[account(
        init,
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

pub fn init_escrow(ctx: Context<InitEscrow>) -> Result<()> {
    // Escrow is created, ready to receive deposits
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
