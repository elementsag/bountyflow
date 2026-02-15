import { Probot } from 'probot';
import { handleBountyCommand, handleClaimCommand, handleStatusCommand } from './commands';
import { BountyFlowClient } from './solana';

const client = new BountyFlowClient();

export = (app: Probot) => {
  app.on('issue_comment.created', async (context) => {
    const comment = context.payload.comment.body.trim();
    const issue = context.payload.issue;
    const repo = context.payload.repository;

    if (comment.startsWith('/bounty')) {
      await handleBountyCommand(context, client, repo, issue);
    } else if (comment === '/claim') {
      await handleClaimCommand(context, client, repo, issue);
    } else if (comment === '/status') {
      await handleStatusCommand(context, client, repo, issue);
    }
  });

  app.on('pull_request.closed', async (context) => {
    const pr = context.payload.pull_request;
    
    if (!pr.merged) return;

    const linkedIssues = extractLinkedIssues(pr.body || '');
    if (linkedIssues.length === 0) return;

    await handlePRMerged(context, client, pr, linkedIssues);
  });
};

function extractLinkedIssues(body: string): number[] {
  const patterns = [
    /(?:fixes|closes|resolves)\s+#(\d+)/gi,
    /#(\d+)/g,
  ];
  
  const issues: Set<number> = new Set();
  
  for (const pattern of patterns) {
    let match;
    while ((match = pattern.exec(body)) !== null) {
      issues.add(parseInt(match[1], 10));
    }
  }
  
  return Array.from(issues);
}

async function handlePRMerged(
  context: any,
  client: BountyFlowClient,
  pr: any,
  issueNumbers: number[]
) {
  const repo = context.payload.repository;
  const owner = repo.owner.login;
  const repoName = repo.name;

  const commits = await context.octokit.pulls.listCommits({
    owner,
    repo: repoName,
    pull_number: pr.number,
  });

  const contributors = new Map<string, number>();
  for (const commit of commits.data) {
    const author = commit.author?.login || commit.commit.author?.name || 'unknown';
    contributors.set(author, (contributors.get(author) || 0) + 1);
  }

  for (const issueNumber of issueNumbers) {
    try {
      await client.releaseBounty(
        repo.id,
        issueNumber,
        Array.from(contributors.entries()).map(([handle, commits]) => ({
          githubHandle: handle,
          commits,
        }))
      );

      await context.octokit.issues.createComment({
        owner,
        repo: repoName,
        issue_number: issueNumber,
        body: `🎉 **Bounty Released!**\n\n${Array.from(contributors.entries())
          .map(([handle, commits]) => `@${handle}: ${commits} commits`)
          .join('\n')}\n\nWithdraw your share: /withdraw`,
      });
    } catch (error) {
      console.error(`Failed to release bounty for issue #${issueNumber}:`, error);
    }
  }
}
