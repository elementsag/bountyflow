use anchor_lang::prelude::*;

#[account]
pub struct BountyAccount {
    pub repo_id: u64,
    pub issue_number: u64,
    pub creator: Pubkey,
    pub amount: u64,
    pub token_mint: Pubkey,
    pub status: BountyStatus,
    pub created_at: i64,
    pub bump: u8,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Eq)]
pub enum BountyStatus {
    Open,
    Funded,
    Closed,
    Released,
}

#[account]
pub struct ClaimAccount {
    pub bounty: Pubkey,
    pub claimant: Pubkey,
    pub github_handle: String,
    pub commits: u32,
    pub share_bps: u32,
    pub withdrawn: bool,
    pub bump: u8,
}

#[account]
pub struct AgentAccount {
    pub github_handle: String,
    pub wallet: Pubkey,
    pub signature: [u8; 64],
    pub registered_at: i64,
    pub bump: u8,
}

#[account]
pub struct TreasuryAccount {
    pub authority: Pubkey,
    pub fee_bps: u64,
    pub bump: u8,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct ContributorInfo {
    pub github_handle: String,
    pub commits: u32,
}

impl BountyAccount {
    pub const LEN: usize = 8 + 8 + 8 + 32 + 8 + 32 + 1 + 8 + 1;
}

impl ClaimAccount {
    pub const LEN: usize = 8 + 32 + 32 + 40 + 4 + 4 + 1 + 1;
}

impl AgentAccount {
    pub const LEN: usize = 8 + 40 + 32 + 64 + 8 + 1;
}

impl TreasuryAccount {
    pub const LEN: usize = 8 + 32 + 8 + 1;
}
