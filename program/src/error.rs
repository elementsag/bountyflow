use anchor_lang::prelude::*;

#[error_code]
pub enum BountyError {
    #[msg("Bounty not funded - no tokens deposited")]
    BountyNotFunded,
    
    #[msg("Bounty already has active claims")]
    BountyAlreadyClaimed,
    
    #[msg("Only bounty creator can perform this action")]
    NotBountyCreator,
    
    #[msg("Cannot cancel bounty with active claims")]
    HasActiveClaims,
    
    #[msg("Unauthorized - only bot authority can call this")]
    NotAuthorized,
    
    #[msg("Invalid signature - wallet ownership not proven")]
    InvalidSignature,
    
    #[msg("Funds already withdrawn")]
    AlreadyWithdrawn,
    
    #[msg("Bounty not yet released")]
    BountyNotReleased,
    
    #[msg("Refund timeout not reached - wait 30 days")]
    TimeoutNotReached,
    
    #[msg("No commits allocated to this claimant")]
    NoCommitsAllocated,
    
    #[msg("Agent not registered")]
    AgentNotRegistered,
    
    #[msg("Invalid token amount")]
    InvalidAmount,
    
    #[msg("Bounty already exists for this issue")]
    BountyAlreadyExists,
}
