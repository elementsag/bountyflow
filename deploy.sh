#!/bin/bash
set -e

echo "🦀 BountyFlow Deployment Script"
echo "================================"

# Check prerequisites
if ! command -v solana &> /dev/null; then
    echo "❌ Solana CLI not installed"
    echo "Run: sh -c \"\$(curl -sSfL https://release.anza.xyz/stable/install)\""
    exit 1
fi

if ! command -v anchor &> /dev/null; then
    echo "❌ Anchor not installed"
    echo "Run: cargo install --git https://github.com/coral-xyz/anchor avm --locked --force && avm install 0.29.0"
    exit 1
fi

# Configure for devnet
echo "🌐 Configuring Solana for devnet..."
solana config set --url devnet

# Check balance (handle errors gracefully)
echo "💰 Checking balance..."
BALANCE=$(solana balance 2>/dev/null | awk '{print $1}' || echo "0")
echo "   Current balance: $BALANCE SOL"

# Try airdrop if balance is low (skip if network error)
if [ "$BALANCE" = "0" ] || [ "$BALANCE" = "" ]; then
    echo "💸 Attempting airdrop..."
    solana airdrop 2 2>/dev/null || echo "⚠️  Airdrop failed (network issue or rate limited)"
fi

# Navigate to program directory
cd program 2>/dev/null || { echo "❌ program/ directory not found"; exit 1; }

# Build program
echo "🔨 Building Anchor program..."
anchor build 2>&1 || { echo "❌ Build failed"; exit 1; }

# Get program ID
PROGRAM_ID=$(anchor keys list 2>/dev/null | grep bountyflow | awk '{print $2}' || echo "")
if [ -z "$PROGRAM_ID" ]; then
    echo "⚠️  Could not get program ID from anchor keys list"
    echo "   Using default placeholder..."
    PROGRAM_ID="Bount11111111111111111111111111111111111111"
fi
echo "📝 Program ID: $PROGRAM_ID"

# Update program ID in files (if sed supports -i)
if command -v sed &> /dev/null; then
    echo "📝 Updating program ID in source files..."
    sed -i "s/declare_id!.*/declare_id!(\"$PROGRAM_ID\");/" src/lib.rs 2>/dev/null || true
fi

# Deploy
echo "🚀 Deploying to devnet..."
anchor deploy --provider.cluster devnet 2>&1

cd ..

echo ""
echo "✅ Deployment Complete!"
echo "======================="
echo "Program ID: $PROGRAM_ID"
echo "Explorer: https://explorer.solana.com/address/$PROGRAM_ID?cluster=devnet"
echo ""
echo "Next steps:"
echo "1. Update PROGRAM_ID in bot/.env"
echo "2. cd bot && npm install && npm start"
