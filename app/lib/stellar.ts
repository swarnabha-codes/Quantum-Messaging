import { Keypair, TransactionBuilder, Networks, Memo, Operation, Horizon, Asset } from '@stellar/stellar-sdk';

const server = new Horizon.Server('https://horizon-testnet.stellar.org');

export function generateKeypair(): { publicKey: string, secretKey: string } {
  const keypair = Keypair.random();
  return {
    publicKey: keypair.publicKey(),
    secretKey: keypair.secret(),
  };
}

export async function fundAccount(publicKey: string): Promise<void> {
  const response = await fetch(`https://friendbot.stellar.org?addr=${publicKey}`);
  if (!response.ok) {
    throw new Error('Failed to fund account');
  }
}

export async function sendTransaction(
  secretKey: string,
  destination: string,
  memoData: string
): Promise<string> {
  const keypair = Keypair.fromSecret(secretKey);
  const account = await server.loadAccount(keypair.publicKey());

  const transaction = new TransactionBuilder(account, {
    fee: '100',
    networkPassphrase: Networks.TESTNET,
  })
    .addOperation(Operation.payment({
      destination,
      asset: Asset.native(),
      amount: '0.0001', // Minimal amount
    }))
    .addMemo(Memo.text(memoData)) // Use text memo
    .setTimeout(30)
    .build();

  transaction.sign(keypair);
  const result = await server.submitTransaction(transaction);
  return result.hash;
}

export async function getTransactions(publicKey: string): Promise<any[]> {
  const transactions = await server.transactions()
    .forAccount(publicKey)
    .limit(10)
    .call();
  return transactions.records;
}