# BountyFlow Agent Instructions

This file provides instructions for AI agents working on the BountyFlow codebase or using BountyFlow to earn bounties.

## Project Overview

BountyFlow is a decentralized GitHub bounty system on Solana. It enables:
- Creating bounties on GitHub issues
- Claiming and solving issues
- Automatic payout on PR merge
- Split rewards among contributors

## Architecture Principles

1. **Trustless Escrow**: Funds held in Solana PDAs, not centralized
2. **Automatic Release**: PR merge triggers payout, no manual approval
3. **Commit-Based Split**: Multiple contributors split by commit count
4. **Protocol Fee**: 2.5% fee on SPL token bounties (SOL is free)

## Code Structure

```
/program
  /src
    lib.rs        - Program entry point, instruction routing
    bounty.rs     - Bounty creation, funding, cancellation
    claim.rs      - Claiming, releasing, withdrawing
    agent.rs      - Agent identity registration
    escrow.rs     - Token escrow operations
    state.rs      - Account definitions
    error.rs      - Custom error codes
```

## Core Flows

### Create Bounty Flow

```
1. User comments /bounty 10 USDC on GitHub issue
2. Bot calls create_bounty instruction
3. Program creates BountyAccount PDA
4. Bot replies with deposit address (PDA)
5. User transfers 10 USDC to PDA
6. Bot detects deposit, updates status to Funded
```

### Solve & Earn Flow

```
1. Agent scans for funded bounties
2. Agent comments /claim on issue
3. Program creates ClaimAccount for agent
4. Agent forks repo, creates PR
5. PR merged, linked to issue
6. Bot calls release_on_merge with contributors
7. Program splits bounty by commits
8. Agent calls withdraw to get tokens
```

### Fee Calculation

```rust
// 2.5% fee for SPL tokens, 0% for SOL
fn calculate_fees(amount: u64, is_native_sol: bool) -> (u64, u64) {
    if is_native_sol {
        (0, amount)  // No fee, full amount to solvers
    } else {
        let fee = amount * 25 / 1000;  // 2.5%
        let payout = amount - fee;
        (fee, payout)
    }
}

// Example: 100 USDC bounty
// Fee: 2.5 USDC → Treasury
// Payout: 97.5 USDC → Solvers
```

## Key Dependencies

### Solana/Anchor
- `anchor-lang` - Program framework
- `anchor-spl` - Token operations
- `solana-program` - Low-level operations

### Backend
- `@solana/web3.js` - Solana RPC
- `@coral-xyz/anchor` - Anchor client
- `@octokit/rest` - GitHub API
- `probot` - GitHub Bot framework

## Account Relationships

```
RepoAccount (PDA)
    │
    ├── BountyAccount #1 (PDA)
    │       │
    │       ├── ClaimAccount (Agent A)
    │       ├── ClaimAccount (Agent B)
    │       └── Token Escrow (PDA)
    │
    └── BountyAccount #2 (PDA)
            │
            └── ...

AgentAccount (PDA) ← Maps GitHub handle to Solana wallet
```

## Instruction Reference

### create_bounty

```rust
pub fn create_bounty(
    ctx: Context<CreateBounty>,
    repo_id: u64,
    issue_number: u64,
    amount: u64,
    token_mint: Pubkey,
) -> Result<()>
```

Creates a new bounty account. Does NOT transfer tokens - must call `deposit` after.

**Accounts:**
- `[signer]` Creator's wallet
- `[writable]` Bounty PDA (created)
- `[writable]` Treasury token account (for fees)
- `[]` Token mint
- `[]` System program
- `[]` Rent sysvar

### deposit

```rust
pub fn deposit(ctx: Context<Deposit>, amount: u64) -> Result<()>
```

Transfers tokens from creator to bounty escrow.

**Accounts:**
- `[signer]` Creator's wallet
- `[writable]` Creator's token account
- `[writable]` Bounty escrow (PDA)
- `[writable]` Bounty account
- `[]` Token program

### claim_bounty

```rust
pub fn claim_bounty(
    ctx: Context<ClaimBounty>,
    github_handle: String,
) -> Result<()>
```

Registers intent to solve. Must have registered agent identity first.

**Accounts:**
- `[signer]` Claimant's wallet
- `[writable]` Claim PDA (created)
- `[]` Bounty account
- `[]` Agent account (must exist)
- `[]` System program

### release_on_merge

```rust
pub struct ContributorInfo {
    pub github_handle: String,
    pub commits: u32,
}

pub fn release_on_merge(
    ctx: Context<ReleaseOnMerge>,
    contributors: Vec<ContributorInfo>,
) -> Result<()>
```

Called by bot when PR is merged. Calculates and allocates shares.

**Fee Calculation:**
```rust
// For non-SOL tokens, deduct 2.5%
let (fee, payout) = calculate_fees(bounty.amount, is_native);

// Fee goes to treasury
token_transfer(fee, bounty_escrow, treasury)?;

// Payout split among contributors
for contributor in contributors {
    let share = payout * contributor.commits / total_commits;
    allocate_share(contributor.wallet, share);
}
```

**Accounts:**
- `[signer]` Bot authority (whitelisted)
- `[writable]` Bounty account
- `[writable]` Bounty escrow
- `[writable]` Treasury token account
- `[]` Token program
- `[]` All claim accounts

### withdraw

```rust
pub fn withdraw(ctx: Context<Withdraw>) -> Result<()>
```

Transfers allocated share to claimant's wallet.

**Accounts:**
- `[signer]` Claimant's wallet
- `[writable]` Claim account
- `[writable]` Bounty escrow
- `[writable]` Claimant's token account
- `[]` Token program

## Error Handling

```rust
#[msg("Bounty not funded")]
BountyNotFunded,

#[msg("Bounty already has active claims")]
BountyAlreadyClaimed,

#[msg("Only bounty creator can cancel")]
NotBountyCreator,

#[msg("Cannot cancel bounty with active claims")]
HasActiveClaims,

#[msg("Unauthorized - only bot can call this")]
NotAuthorized,

#[msg("Invalid signature - wallet ownership not proven")]
InvalidSignature,

#[msg("Funds already withdrawn")]
AlreadyWithdrawn,

#[msg("Bounty not yet released")]
BountyNotReleased,

#[msg("Refund timeout not reached")]
TimeoutNotReached,

#[msg("No commits allocated to this claimant")]
NoCommitsAllocated,
```

## Testing

### Unit Tests

```bash
cd program
anchor test
```

### Integration Tests

```typescript
describe('BountyFlow', () => {
    it('creates, funds, claims, releases, and withdraws', async () => {
        // Create bounty
        await program.methods.createBounty(repoId, issue, 10_000_000, USDC_MINT)
            .accounts({ creator: creatorWallet })
            .rpc();
        
        // Deposit
        await program.methods.deposit(10_000_000)
            .accounts({ /* ... */ })
            .rpc();
        
        // Claim
        await program.methods.claimBounty('solver-handle')
            .accounts({ /* ... */ })
            .rpc();
        
        // Release (as bot)
        await program.methods.releaseOnMerge([
            { githubHandle: 'solver-handle', commits: 3 }
        ]).accounts({ /* ... */ }).rpc();
        
        // Withdraw
        await program.methods.withdraw()
            .accounts({ /* ... */ })
            .rpc();
    });
});
```

## Environment Variables

### Program
```
ANCHOR_PROVIDER_URL=https://api.devnet.solana.com
ANCHOR_WALLET=~/.config/solana/id.json
```

### API
```
DATABASE_URL=postgresql://user:pass@localhost/bountyflow
SOLANA_RPC_URL=https://api.devnet.solana.com
PROGRAM_ID=BountY11111111111111111111111111111111111
BOT_PUBLIC_KEY=<bot-wallet-pubkey>
TREASURY_WALLET=<treasury-pubkey>
FEE_BPS=250  // 2.5% = 250 basis points
```

### Bot
```
APP_ID=<github-app-id>
PRIVATE_KEY="-----BEGIN RSA PRIVATE KEY-----..."
WEBHOOK_SECRET=<webhook-secret>
API_URL=http://localhost:3000
```

## Common Tasks

### Add New Token Support

1. Add mint address to `TOKEN_MINTS` constant
2. Bot auto-detects decimals from mint
3. No code changes needed

### Adjust Fee Percentage

1. Update `FEE_BPS` in program constants
2. Recompile and redeploy program
3. Update API environment variable

### Add New Command

1. Create handler in `bot/src/commands.ts`
2. Register in `bot/src/index.ts`
3. Add help text in `/help` handler

## Deployment

### Devnet

```bash
# Build
anchor build

# Deploy
anchor deploy --provider.cluster devnet

# Initialize
anchor run init

# Verify
solana program show <program-id>
```

### Mainnet

```bash
# Build with mainnet features
anchor build -- --features mainnet

# Deploy (requires SOL)
anchor deploy --provider.cluster mainnet

# Verify on explorer
# https://explorer.solana.com/address/<program-id>
```

## Monitoring

### Key Metrics

- Total bounties created
- Total value locked (TVL)
- Total paid out
- Active claims
- Fee revenue

### Health Checks

```typescript
// API health
GET /health → { status: "ok", program: "active", db: "connected" }

// Program health
connection.getAccountInfo(programId) → not null
```

## Security Checklist

- [ ] All PDAs use canonical bump
- [ ] Token transfers use SPL Token program
- [ ] Authority checked before every instruction
- [ ] No unchecked math operations
- [ ] Webhook signatures verified
- [ ] SQL injection prevented (parameterized queries)
- [ ] Rate limiting on all endpoints
- [ ] Private keys never logged
