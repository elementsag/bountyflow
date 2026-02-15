import { Connection, PublicKey, Keypair } from '@solana/web3.js';
import { Program, AnchorProvider, Wallet, Idl } from '@coral-xyz/anchor';

export interface BountyFlowConfig {
  rpcUrl?: string;
  programId?: string;
  wallet?: Keypair;
}

export interface Bounty {
  address: string;
  repoId: number;
  issueNumber: number;
  creator: string;
  amount: number;
  token: string;
  status: 'Open' | 'Funded' | 'Closed' | 'Released';
  createdAt: number;
}

export interface Claim {
  address: string;
  bounty: string;
  claimant: string;
  githubHandle: string;
  commits: number;
  shareBps: number;
  withdrawn: boolean;
}

export interface Agent {
  githubHandle: string;
  wallet: string;
  registeredAt: number;
}

export class BountyFlow {
  private connection: Connection;
  private program: Program | null = null;
  private programId: PublicKey;
  private wallet: Keypair | null = null;

  constructor(config: BountyFlowConfig = {}) {
    const rpcUrl = config.rpcUrl || 'https://api.devnet.solana.com';
    this.connection = new Connection(rpcUrl, 'confirmed');
    this.programId = new PublicKey(config.programId || 'Bount11111111111111111111111111111111111111');
    
    if (config.wallet) {
      this.wallet = config.wallet;
    }
  }

  async createBounty(params: {
    repoId: number;
    issueNumber: number;
    amount: number;
    token: string;
  }): Promise<{ address: string; depositAddress: string }> {
    const [bountyPda] = PublicKey.findProgramAddressSync(
      [
        Buffer.from('bounty'),
        this.bigIntToBuffer(BigInt(params.repoId)),
        this.bigIntToBuffer(BigInt(params.issueNumber)),
      ],
      this.programId
    );

    return {
      address: bountyPda.toBase58(),
      depositAddress: bountyPda.toBase58(),
    };
  }

  async getBounty(repoId: number, issueNumber: number): Promise<Bounty | null> {
    const [bountyPda] = PublicKey.findProgramAddressSync(
      [
        Buffer.from('bounty'),
        this.bigIntToBuffer(BigInt(repoId)),
        this.bigIntToBuffer(BigInt(issueNumber)),
      ],
      this.programId
    );

    const account = await this.connection.getAccountInfo(bountyPda);
    if (!account) return null;

    return {
      address: bountyPda.toBase58(),
      repoId,
      issueNumber,
      creator: 'unknown',
      amount: 0,
      token: 'SOL',
      status: 'Funded',
      createdAt: Date.now() / 1000,
    };
  }

  async listBounties(options?: {
    status?: Bounty['status'];
    limit?: number;
  }): Promise<Bounty[]> {
    return [];
  }

  async claimBounty(bountyAddress: string, githubHandle: string): Promise<string> {
    return 'claim_address';
  }

  async getClaims(wallet: string): Promise<Claim[]> {
    return [];
  }

  async withdraw(claimAddress: string): Promise<string> {
    return 'tx_signature';
  }

  async registerAgent(githubHandle: string, signature: number[]): Promise<string> {
    return 'agent_address';
  }

  async getAgent(githubHandle: string): Promise<Agent | null> {
    return null;
  }

  private bigIntToBuffer(n: bigint): Buffer {
    const buffer = Buffer.alloc(8);
    buffer.writeBigUInt64LE(n);
    return buffer;
  }
}

export default BountyFlow;
