import { Connection, PublicKey, Keypair } from '@solana/web3.js';
import { Program, AnchorProvider, Wallet } from '@coral-xyz/anchor';
import { TOKEN_MINTS } from './tokens';

interface CreateBountyParams {
  repoId: number;
  issueNumber: number;
  amount: number;
  token: string;
}

interface ClaimParams {
  repoId: number;
  issueNumber: number;
  githubHandle: string;
}

interface Contributor {
  githubHandle: string;
  commits: number;
}

export class BountyFlowClient {
  private connection: Connection;
  private program: Program | null = null;
  private wallet: Keypair | null = null;

  constructor() {
    const rpcUrl = process.env.SOLANA_RPC_URL || 'https://api.devnet.solana.com';
    this.connection = new Connection(rpcUrl, 'confirmed');
    
    if (process.env.BOT_WALLET) {
      this.wallet = Keypair.fromSecretKey(
        Buffer.from(process.env.BOT_WALLET, 'base58')
      );
    }
  }

  async createBounty(params: CreateBountyParams): Promise<{ depositAddress: string }> {
    const tokenMint = TOKEN_MINTS[params.token as keyof typeof TOKEN_MINTS];
    if (!tokenMint) {
      throw new Error(`Unsupported token: ${params.token}`);
    }

    const [bountyPda] = PublicKey.findProgramAddressSync(
      [
        Buffer.from('bounty'),
        Buffer.from(params.repoId.toString()),
        Buffer.from(params.issueNumber.toString()),
      ],
      new PublicKey(process.env.PROGRAM_ID!)
    );

    return {
      depositAddress: bountyPda.toBase58(),
    };
  }

  async getBounty(repoId: number, issueNumber: number): Promise<any> {
    const [bountyPda] = PublicKey.findProgramAddressSync(
      [
        Buffer.from('bounty'),
        Buffer.from(repoId.toString()),
        Buffer.from(issueNumber.toString()),
      ],
      new PublicKey(process.env.PROGRAM_ID!)
    );

    const account = await this.connection.getAccountInfo(bountyPda);
    if (!account) {
      return null;
    }

    return {
      address: bountyPda.toBase58(),
      status: 'Funded',
      amount: 10,
      token: 'SOL',
      createdAt: Date.now() / 1000,
      claims: [],
    };
  }

  async getAgent(githubHandle: string): Promise<{ wallet: string } | null> {
    const [agentPda] = PublicKey.findProgramAddressSync(
      [Buffer.from('agent'), Buffer.from(githubHandle)],
      new PublicKey(process.env.PROGRAM_ID!)
    );

    const account = await this.connection.getAccountInfo(agentPda);
    if (!account) {
      return null;
    }

    return {
      wallet: '7xKXtg31CW42R3TKA7K2xXVQ9zQbQ9Z9Z9Z9Z9Z9Z9Z9',
    };
  }

  async claimBounty(params: ClaimParams): Promise<void> {
    console.log('Claiming bounty:', params);
  }

  async releaseBounty(
    repoId: number,
    issueNumber: number,
    contributors: Contributor[]
  ): Promise<void> {
    console.log('Releasing bounty:', { repoId, issueNumber, contributors });
  }

  async withdraw(claimAddress: string): Promise<string> {
    return 'tx_signature';
  }
}
