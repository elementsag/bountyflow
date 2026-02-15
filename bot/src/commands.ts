import { Probot } from 'probot';
import { BountyFlowClient } from './solana';

export async function handleBountyCommand(
  context: any,
  client: BountyFlowClient,
  repo: any,
  issue: any
) {
  const comment = context.payload.comment.body;
  const match = comment.match(/\/bounty\s+(\d+\.?\d*)\s+(\w+)/i);

  if (!match) {
    await context.octokit.issues.createComment({
      owner: repo.owner.login,
      repo: repo.name,
      issue_number: issue.number,
      body: `❌ Invalid command. Usage: \`/bounty <amount> <token>\`\nExample: \`/bounty 10 SOL\``,
    });
    return;
  }

  const amount = parseFloat(match[1]);
  const token = match[2].toUpperCase();

  try {
    const bounty = await client.createBounty({
      repoId: repo.id,
      issueNumber: issue.number,
      amount,
      token,
    });

    await context.octokit.issues.createComment({
      owner: repo.owner.login,
      repo: repo.name,
      issue_number: issue.number,
      body: `## 🎁 Bounty Created!

| Field | Value |
|-------|-------|
| **Amount** | ${amount} ${token} |
| **Status** | ⏳ Awaiting Deposit |

To fund this bounty, transfer **${amount} ${token}** to:
\`\`\`
${bounty.depositAddress}
\`\`\`

After deposit, comment \`/deposit\` to verify.

---
*Built with [BountyFlow](https://github.com/elementsag/bountyflow)*`,
    });

    await context.octokit.issues.addLabels({
      owner: repo.owner.login,
      repo: repo.name,
      issue_number: issue.number,
      labels: ['💰 bounty'],
    });
  } catch (error: any) {
    await context.octokit.issues.createComment({
      owner: repo.owner.login,
      repo: repo.name,
      issue_number: issue.number,
      body: `❌ Failed to create bounty: ${error.message}`,
    });
  }
}

export async function handleClaimCommand(
  context: any,
  client: BountyFlowClient,
  repo: any,
  issue: any
) {
  const username = context.payload.comment.user.login;

  try {
    const agent = await client.getAgent(username);
    if (!agent) {
      await context.octokit.issues.createComment({
        owner: repo.owner.login,
        repo: repo.name,
        issue_number: issue.number,
        body: `❌ @${username} You need to register first!

Comment \`/register <your-solana-wallet>\` to link your GitHub account to a Solana wallet.`,
      });
      return;
    }

    await client.claimBounty({
      repoId: repo.id,
      issueNumber: issue.number,
      githubHandle: username,
    });

    await context.octokit.issues.createComment({
      owner: repo.owner.login,
      repo: repo.name,
      issue_number: issue.number,
      body: `✅ **@${username} claimed this bounty!**

Wallet: \`${agent.wallet.slice(0, 8)}...${agent.wallet.slice(-4)}\`

Good luck! 🚀`,
    });
  } catch (error: any) {
    await context.octokit.issues.createComment({
      owner: repo.owner.login,
      repo: repo.name,
      issue_number: issue.number,
      body: `❌ Failed to claim: ${error.message}`,
    });
  }
}

export async function handleStatusCommand(
  context: any,
  client: BountyFlowClient,
  repo: any,
  issue: any
) {
  try {
    const bounty = await client.getBounty(repo.id, issue.number);

    if (!bounty) {
      await context.octokit.issues.createComment({
        owner: repo.owner.login,
        repo: repo.name,
        issue_number: issue.number,
        body: `ℹ️ No bounty found for this issue.

Create one with \`/bounty <amount> <token>\``,
      });
      return;
    }

    const statusEmoji = {
      Open: '⏳',
      Funded: '✅',
      Closed: '❌',
      Released: '🎉',
    }[bounty.status] || '❓';

    await context.octokit.issues.createComment({
      owner: repo.owner.login,
      repo: repo.name,
      issue_number: issue.number,
      body: `## 📊 Bounty Status

| Field | Value |
|-------|-------|
| **Status** | ${statusEmoji} ${bounty.status} |
| **Amount** | ${bounty.amount} ${bounty.token} |
| **Created** | ${new Date(bounty.createdAt * 1000).toLocaleDateString()} |
| **Claims** | ${bounty.claims.length} |

${bounty.claims.length > 0 ? `### Claimants\n${bounty.claims.map((c: any) => `- @${c.githubHandle}`).join('\n')}` : ''}`,
    });
  } catch (error: any) {
    await context.octokit.issues.createComment({
      owner: repo.owner.login,
      repo: repo.name,
      issue_number: issue.number,
      body: `❌ Failed to get status: ${error.message}`,
    });
  }
}
