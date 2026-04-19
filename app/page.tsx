'use client';

import { useState, useEffect } from 'react';
import { BB84 } from './lib/bb84';
import { encryptMessage, decryptMessage } from './lib/crypto';
import { generateKeypair, fundAccount, sendTransaction, getTransactions } from './lib/stellar';

export default function Home() {
  const [keypair, setKeypair] = useState<{ publicKey: string, secretKey: string } | null>(null);
  const [recipient, setRecipient] = useState('');
  const [message, setMessage] = useState('');
  const [sharedKey, setSharedKey] = useState('');
  const [generatedKey, setGeneratedKey] = useState('');
  const [transactions, setTransactions] = useState<any[]>([]);
  const [loading, setLoading] = useState(false);

  useEffect(() => {
    const stored = localStorage.getItem('stellarKeypair');
    if (stored) {
      setKeypair(JSON.parse(stored));
    }
  }, []);

  const createAccount = async () => {
    const kp = generateKeypair();
    setKeypair(kp);
    localStorage.setItem('stellarKeypair', JSON.stringify(kp));
    try {
      await fundAccount(kp.publicKey);
      alert('Account created and funded!');
    } catch (error) {
      alert('Account created, but funding failed. Please fund manually.');
    }
  };

  const generateQuantumKey = () => {
    const bb84 = new BB84(256);
    const key = bb84.simulateKeyExchange();
    setGeneratedKey(key);
  };

  const sendMessage = async () => {
    if (!keypair || !recipient || !message || !generatedKey) return;

    setLoading(true);
    try {
      const encrypted = encryptMessage(message, generatedKey);
      // Limit to 28 chars for text memo
      const memoData = encrypted.slice(0, 28);
      await sendTransaction(keypair.secretKey, recipient, memoData);
      alert('Message sent!');
      setMessage('');
      setGeneratedKey('');
    } catch (error) {
      alert('Failed to send message: ' + error);
    }
    setLoading(false);
  };

  const loadTransactions = async () => {
    if (!keypair) return;
    try {
      const txns = await getTransactions(keypair.publicKey);
      setTransactions(txns);
    } catch (error) {
      alert('Failed to load transactions: ' + error);
    }
  };

  const decryptMemo = (memo: string) => {
    if (!sharedKey) return 'Enter shared key to decrypt';
    try {
      return decryptMessage(memo, sharedKey);
    } catch {
      return 'Decryption failed';
    }
  };

  return (
    <div className="min-h-screen bg-gray-100 p-8">
      <div className="max-w-4xl mx-auto">
        <h1 className="text-3xl font-bold mb-8">Stellar Quantum Messaging</h1>

        {!keypair ? (
          <div className="bg-white p-6 rounded-lg shadow">
            <h2 className="text-xl mb-4">Create Stellar Account</h2>
            <button
              onClick={createAccount}
              className="bg-blue-500 text-white px-4 py-2 rounded"
            >
              Create Account
            </button>
          </div>
        ) : (
          <div className="space-y-8">
            <div className="bg-white p-6 rounded-lg shadow">
              <h2 className="text-xl mb-4">Your Account</h2>
              <p>Public Key: {keypair.publicKey}</p>
            </div>

            <div className="bg-white p-6 rounded-lg shadow">
              <h2 className="text-xl mb-4">Generate Quantum Key</h2>
              <button
                onClick={generateQuantumKey}
                className="bg-green-500 text-white px-4 py-2 rounded mb-4"
              >
                Generate Key
              </button>
              {generatedKey && (
                <div>
                  <p className="mb-2">Share this key with recipient:</p>
                  <textarea
                    value={generatedKey}
                    readOnly
                    className="w-full p-2 border rounded"
                    rows={3}
                  />
                </div>
              )}
            </div>

            <div className="bg-white p-6 rounded-lg shadow">
              <h2 className="text-xl mb-4">Send Message</h2>
              <input
                type="text"
                placeholder="Recipient Public Key"
                value={recipient}
                onChange={(e) => setRecipient(e.target.value)}
                className="w-full p-2 border rounded mb-4"
              />
              <textarea
                placeholder="Message (short)"
                value={message}
                onChange={(e) => setMessage(e.target.value)}
                className="w-full p-2 border rounded mb-4"
                rows={3}
              />
              <button
                onClick={sendMessage}
                disabled={loading || !generatedKey}
                className="bg-purple-500 text-white px-4 py-2 rounded disabled:opacity-50"
              >
                {loading ? 'Sending...' : 'Send Encrypted Message'}
              </button>
            </div>

            <div className="bg-white p-6 rounded-lg shadow">
              <h2 className="text-xl mb-4">Received Messages</h2>
              <input
                type="text"
                placeholder="Shared Key for Decryption"
                value={sharedKey}
                onChange={(e) => setSharedKey(e.target.value)}
                className="w-full p-2 border rounded mb-4"
              />
              <button
                onClick={loadTransactions}
                className="bg-orange-500 text-white px-4 py-2 rounded mb-4"
              >
                Load Transactions
              </button>
              <div className="space-y-2">
                {transactions.map((tx) => (
                  <div key={tx.id} className="border p-4 rounded">
                    <p>From: {tx.source_account}</p>
                    <p>Memo: {tx.memo}</p>
                    <p>Decrypted: {decryptMemo(tx.memo)}</p>
                  </div>
                ))}
              </div>
            </div>
          </div>
        )}
      </div>
    </div>
  );
}
