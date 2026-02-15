use anchor_lang::prelude::*;

#[account]
pub struct BountyAccount {
    pub repo_id: u64,        // 8
    pub issue_number: u64,   // 8
    pub creator: Pubkey,     // 32
    pub amount: u64,         // 8
    pub token_mint: Pubkey,  // 32
    pub status: BountyStatus, // 1
    pub created_at: i64,     // 8
    pub bump: u8,            // 1
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, PartialEq, Eq)]
pub enum BountyStatus {
    Open,
    Funded,
    Closed,
    Released,
}

#[account]
pub struct ClaimAccount {
    pub bounty: Pubkey,          // 32
    pub claimant: Pubkey,        // 32
    pub github_handle: String,   // 4 + max 39 (GitHub username limit)
    pub commits: u32,            // 4
    pub share_bps: u32,          // 4
    pub withdrawn: bool,         // 1
    pub bump: u8,                // 1
}

#[account]
pub struct AgentAccount {
    pub github_handle: String,   // 4 + max 39
    pub wallet: Pubkey,          // 32
    pub signature: [u8; 64],     // 64
    pub registered_at: i64,      // 8
    pub bump: u8,                // 1
}

#[account]
pub struct TreasuryAccount {
    pub authority: Pubkey,       // 32
    pub fee_bps: u64,            // 8
    pub bump: u8,                // 1
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct ContributorInfo {
    pub github_handle: String,
    pub commits: u32,
}

// Discriminator: 8 bytes
impl BountyAccount {
    pub const LEN: usize = 8 + 8 + 8 + 32 + 8 + 32 + 1 + 8 + 1; // 98
}

impl ClaimAccount {
    pub const LEN: usize = 8 + 32 + 32 + (4 + 39) + 4 + 4 + 1 + 1; // 113
}

impl AgentAccount {
    pub const LEN: usize = 8 + (4 + 39) + 32 + 64 + 8 + 1; // 148
}

impl TreasuryAccount {
    pub const LEN: usize = 8 + 32 + 8 + 1; // 49
}
