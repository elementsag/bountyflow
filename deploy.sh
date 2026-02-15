#!/bin/bash
set -e

echo "🦀 BountyFlow Deployment Script"
echo "================================"

# Check prerequisites
command -v solana >/dev/null 2>&1 || { echo "❌ Solana CLI not installed. Run: sh -c \"\$(curl -sSfL https://release.anza.xyz/stable/install)\""; exit 1; }
command -v anchor >/dev/null 2>&1 || { echo "❌ Anchor not installed. Run: cargo install --git https://github.com/coral-xyz/anchor avm --locked --force"; exit 1; }

# Configure for devnet
echo "🌐 Configuring Solana for devnet..."
solana config set --url devnet

# Check balance
BALANCE=$(solana balance | awk '{print $1}')
echo "💰 Current balance: $BALANCE SOL"

if [ $(echo "$BALANCE < 1" | bc -l) ]; then
    echo "💸 Airdropping 2 SOL..."
    solana airdrop 2
fi

# Build program
echo "🔨 Building Anchor program..."
cd program
anchor build
cd ..

# Get program ID
PROGRAM_ID=$(anchor keys list 2>/dev/null | grep bountyflow | awk '{print $2}' || echo "Bount11111111111111111111111111111111111111")
echo "📝 Program ID: $PROGRAM_ID"

# Update program ID in files
echo "📝 Updating program ID in source files..."
sed -i "s/declare_id!.*/declare_id!(\"$PROGRAM_ID\");/" program/src/lib.rs 2>/dev/null || true
sed -i "s/bountyflow = .*/bountyflow = \"$PROGRAM_ID\"/" program/Anchor.toml 2>/dev/null || true

# Deploy
echo "🚀 Deploying to devnet..."
cd program
anchor deploy --provider.cluster devnet
cd ..

echo ""
echo "✅ Deployment Complete!"
echo "======================="
echo "Program ID: $PROGRAM_ID"
echo "Explorer: https://explorer.solana.com/address/$PROGRAM_ID?cluster=devnet"
echo ""
echo "Next steps:"
echo "1. Update PROGRAM_ID in bot/.env"
echo "2. Update NEXT_PUBLIC_PROGRAM_ID in web/.env.local"
echo "3. Deploy bot: cd bot && npm install && npm start"
echo "4. Deploy web: cd web && npm install && npx vercel --prod"
