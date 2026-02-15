# Deployment Guide

This guide covers deploying BountyFlow to Solana devnet/mainnet.

## Prerequisites

### Required Software

| Software | Version | Install Command |
|----------|---------|-----------------|
| Solana CLI | 1.17+ | `sh -c "$(curl -sSfL https://release.anza.xyz/stable/install)"` |
| Anchor | 0.29.0 | `cargo install --git https://github.com/coral-xyz/anchor avm --locked --force && avm install 0.29.0` |
| Node.js | 18+ | `nvm install 18 && nvm use 18` |
| npm/yarn | latest | `npm install -g npm@latest` |

### System Requirements

- **OS**: Linux (Ubuntu 20.04+), macOS, or Windows (WSL)
- **RAM**: 8GB+ recommended
- **Storage**: 20GB+ for Solana tooling
- **Network**: Stable internet connection

## Quick Deploy

```bash
# Clone repo
git clone https://github.com/elementsag/bountyflow.git
cd bountyflow

# Make deploy script executable
chmod +x deploy.sh

# Run deployment
./deploy.sh
```

## Manual Deployment

### 1. Configure Solana

```bash
# Set to devnet (or mainnet-beta for production)
solana config set --url devnet

# Generate keypair if needed
solana-keygen new --no-bip39-passphrase -o ~/.config/solana/id.json

# Get SOL for deployment fees
solana airdrop 2
```

### 2. Build Program

```bash
cd program

# Build
anchor build

# Get program ID
anchor keys list
# Output: bountyflow: <PROGRAM_ID>
```

### 3. Update Program ID

Replace the placeholder program ID in two files:

**program/src/lib.rs:**
```rust
declare_id!("<YOUR_PROGRAM_ID>");
```

**program/Anchor.toml:**
```toml
[programs.devnet]
bountyflow = "<YOUR_PROGRAM_ID>"
```

### 4. Deploy Program

```bash
# Rebuild with new program ID
anchor build

# Deploy
anchor deploy --provider.cluster devnet
```

Save the output - you'll need the program ID for the bot and web app.

### 5. Initialize Treasury

Create `scripts/init.ts`:

```typescript
import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { Bountyflow } from "../target/types/bountyflow";
import { Keypair } from "@solana/web3.js";

async function main() {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);
  
  const program = anchor.workspace.Bountyflow as Program<Bountyflow>;
  const treasury = Keypair.generate();
  
  await program.methods
    .initializeTreasury(new anchor.BN(250)) // 2.5% fee
    .accounts({
      treasury: treasury.publicKey,
      authority: provider.wallet.publicKey,
    })
    .signers([treasury])
    .rpc();
    
  console.log("Treasury initialized:", treasury.publicKey.toBase58());
}

main();
```

Run:
```bash
anchor run init
```

### 6. Deploy GitHub Bot

```bash
cd bot
npm install

# Create environment file
cat > .env << EOF
# GitHub App
APP_ID=<your-github-app-id>
PRIVATE_KEY="-----BEGIN RSA PRIVATE KEY-----
<your-private-key>
-----END RSA PRIVATE KEY-----"
WEBHOOK_SECRET=<random-secret-string>

# Solana
SOLANA_RPC_URL=https://api.devnet.solana.com
PROGRAM_ID=<your-program-id>
BOT_WALLET=<base58-encoded-private-key>
TREASURY_WALLET=<treasury-public-key>
EOF

# Start bot
npm start
```

For production, deploy to:
- **Railway**: `railway up`
- **Render**: Connect GitHub repo
- **Fly.io**: `fly deploy`

### 7. Create GitHub App

1. Go to [GitHub Developer Settings](https://github.com/settings/apps)
2. Click **New GitHub App**
3. Configure:
   - **Name**: BountyFlow Bot (or your preferred name)
   - **Homepage URL**: https://github.com/elementsag/bountyflow
   - **Webhook URL**: `https://your-server.com/api/webhook/github`
   - **Webhook secret**: Same as `WEBHOOK_SECRET` in .env
4. Set permissions:
   - **Issues**: Read and write
   - **Pull requests**: Read only
   - **Contents**: Read only
5. **Subscribe to events**:
   - Issues
   - Issue comment
   - Pull request
6. Click **Create GitHub App**
7. **Generate private key** and download
8. **Install app** on repositories you want to enable bounties on

### 8. Deploy Web Dashboard (Optional)

```bash
cd web
npm install

# Create environment file
cat > .env.local << EOF
NEXT_PUBLIC_PROGRAM_ID=<your-program-id>
NEXT_PUBLIC_RPC_URL=https://api.devnet.solana.com
NEXT_PUBLIC_TREASURY=<treasury-public-key>
EOF

# Build
npm run build

# Deploy to Vercel
npx vercel --prod
```

## Environment Variables Reference

### Bot (.env)

| Variable | Description | Example |
|----------|-------------|---------|
| `APP_ID` | GitHub App ID | `123456` |
| `PRIVATE_KEY` | GitHub App private key | `"-----BEGIN RSA..."` |
| `WEBHOOK_SECRET` | Webhook verification secret | `random-string-123` |
| `SOLANA_RPC_URL` | Solana RPC endpoint | `https://api.devnet.solana.com` |
| `PROGRAM_ID` | Deployed program ID | `Bount1...` |
| `BOT_WALLET` | Bot's Solana keypair (base58) | `3J7...` |
| `TREASURY_WALLET` | Treasury public key | `9XZ...` |

### Web (.env.local)

| Variable | Description | Example |
|----------|-------------|---------|
| `NEXT_PUBLIC_PROGRAM_ID` | Deployed program ID | `Bount1...` |
| `NEXT_PUBLIC_RPC_URL` | Solana RPC endpoint | `https://api.devnet.solana.com` |
| `NEXT_PUBLIC_TREASURY` | Treasury public key | `9XZ...` |

## Mainnet Deployment

For mainnet deployment:

### 1. Security Checklist

- [ ] Program audited
- [ ] All tests passing
- [ ] Authority keys secured
- [ ] Treasury multisig setup
- [ ] Fee parameters reviewed
- [ ] Emergency pause mechanism tested

### 2. Deploy to Mainnet

```bash
# Configure for mainnet
solana config set --url mainnet-beta

# Check you have enough SOL (need ~5-10 SOL)
solana balance

# Build with mainnet features
anchor build -- --features mainnet

# Deploy
anchor deploy --provider.cluster mainnet-beta
```

### 3. Post-Deployment

1. **Verify on Explorer**: https://explorer.solana.com/address/<PROGRAM_ID>
2. **Verify on Solscan**: https://solscan.io/account/<PROGRAM_ID>
3. **Test with small amount first**
4. **Monitor first few transactions**

## Troubleshooting

### "Insufficient funds for instruction"
```bash
solana airdrop 2
```

### "Program failed to compile"
- Check Anchor version: `anchor --version`
- Ensure Rust is up to date: `rustup update`

### "Account already exists"
- Use a different keypair or close existing account

### Bot not responding to commands
- Check webhook URL is accessible
- Verify GitHub App permissions
- Check bot logs: `npm run start -- --log-level debug`

## Support

- GitHub Issues: https://github.com/elementsag/bountyflow/issues
- Discord: https://discord.gg/bountyflow
