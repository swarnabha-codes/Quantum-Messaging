#![no_std]

use soroban_sdk::{
    contract, contractimpl, contracttype, log, symbol_short, vec, Address, Bytes, BytesN, Env,
    Map, String, Symbol, Vec,
};

// ─── Storage Keys ────────────────────────────────────────────────────────────

const ADMIN: Symbol = symbol_short!("ADMIN");
const MSG_CTR: Symbol = symbol_short!("MSG_CTR");

// ─── Data Types ──────────────────────────────────────────────────────────────

/// Represents an encrypted message stored on-chain.
#[contracttype]
#[derive(Clone, Debug)]
pub struct EncryptedMessage {
    /// Unique message identifier
    pub id: u64,
    /// Stellar address of the sender
    pub sender: Address,
    /// Stellar address of the recipient
    pub recipient: Address,
    /// AES-encrypted ciphertext (produced off-chain via BB84-derived key)
    pub ciphertext: String,
    /// SHA-256 hash of the plaintext for integrity verification
    pub content_hash: BytesN<32>,
    /// Ledger sequence number when the message was recorded
    pub timestamp: u64,
    /// Whether the recipient has acknowledged receipt
    pub acknowledged: bool,
}

/// Stores per-user statistics.
#[contracttype]
#[derive(Clone, Debug)]
pub struct UserStats {
    pub messages_sent: u64,
    pub messages_received: u64,
    pub last_active: u64,
}

/// Storage key for a single message by ID.
#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    Message(u64),
    UserStats(Address),
    SentMessages(Address),
    ReceivedMessages(Address),
}

// ─── Contract ────────────────────────────────────────────────────────────────

#[contract]
pub struct QuantumMessageRegistry;

#[contractimpl]
impl QuantumMessageRegistry {
    // ── Initialization ───────────────────────────────────────────────────

    /// Initialize the contract with an admin address.
    /// Can only be called once.
    pub fn initialize(env: Env, admin: Address) {
        if env.storage().instance().has(&ADMIN) {
            panic!("contract already initialized");
        }
        admin.require_auth();
        env.storage().instance().set(&ADMIN, &admin);
        env.storage().instance().set(&MSG_CTR, &0u64);

        log!(&env, "Quantum Message Registry initialized");
    }

    // ── Core Messaging ───────────────────────────────────────────────────

    /// Store a quantum-encrypted message on-chain.
    ///
    /// * `sender`       – The authenticated sender address
    /// * `recipient`    – The intended recipient address
    /// * `ciphertext`   – AES ciphertext encrypted with the BB84-derived key
    /// * `content_hash` – SHA-256 hash of the original plaintext
    ///
    /// Returns the unique message ID.
    pub fn send_message(
        env: Env,
        sender: Address,
        recipient: Address,
        ciphertext: String,
        content_hash: BytesN<32>,
    ) -> u64 {
        sender.require_auth();

        // Increment message counter
        let mut counter: u64 = env.storage().instance().get(&MSG_CTR).unwrap_or(0);
        counter += 1;
        env.storage().instance().set(&MSG_CTR, &counter);

        // Build message record
        let message = EncryptedMessage {
            id: counter,
            sender: sender.clone(),
            recipient: recipient.clone(),
            ciphertext,
            content_hash,
            timestamp: env.ledger().sequence() as u64,
            acknowledged: false,
        };

        // Persist message
        env.storage()
            .persistent()
            .set(&DataKey::Message(counter), &message);

        // Track sent messages for sender
        let mut sent: Vec<u64> = env
            .storage()
            .persistent()
            .get(&DataKey::SentMessages(sender.clone()))
            .unwrap_or(vec![&env]);
        sent.push_back(counter);
        env.storage()
            .persistent()
            .set(&DataKey::SentMessages(sender.clone()), &sent);

        // Track received messages for recipient
        let mut received: Vec<u64> = env
            .storage()
            .persistent()
            .get(&DataKey::ReceivedMessages(recipient.clone()))
            .unwrap_or(vec![&env]);
        received.push_back(counter);
        env.storage()
            .persistent()
            .set(&DataKey::ReceivedMessages(recipient.clone()), &received);

        // Update sender stats
        let mut sender_stats: UserStats = env
            .storage()
            .persistent()
            .get(&DataKey::UserStats(sender.clone()))
            .unwrap_or(UserStats {
                messages_sent: 0,
                messages_received: 0,
                last_active: 0,
            });
        sender_stats.messages_sent += 1;
        sender_stats.last_active = env.ledger().sequence() as u64;
        env.storage()
            .persistent()
            .set(&DataKey::UserStats(sender.clone()), &sender_stats);

        // Update recipient stats
        let mut recipient_stats: UserStats = env
            .storage()
            .persistent()
            .get(&DataKey::UserStats(recipient.clone()))
            .unwrap_or(UserStats {
                messages_sent: 0,
                messages_received: 0,
                last_active: 0,
            });
        recipient_stats.messages_received += 1;
        env.storage()
            .persistent()
            .set(&DataKey::UserStats(recipient.clone()), &recipient_stats);

        log!(&env, "Message {} sent from {} to {}", counter, sender, recipient);

        counter
    }

    // ── Acknowledgement ──────────────────────────────────────────────────

    /// Recipient acknowledges receipt & successful decryption of a message.
    pub fn acknowledge_message(env: Env, recipient: Address, message_id: u64) {
        recipient.require_auth();

        let mut message: EncryptedMessage = env
            .storage()
            .persistent()
            .get(&DataKey::Message(message_id))
            .expect("message not found");

        if message.recipient != recipient {
            panic!("only the recipient can acknowledge");
        }

        message.acknowledged = true;
        env.storage()
            .persistent()
            .set(&DataKey::Message(message_id), &message);

        log!(&env, "Message {} acknowledged by {}", message_id, recipient);
    }

    // ── Queries ──────────────────────────────────────────────────────────

    /// Retrieve a message by its ID.
    pub fn get_message(env: Env, message_id: u64) -> EncryptedMessage {
        env.storage()
            .persistent()
            .get(&DataKey::Message(message_id))
            .expect("message not found")
    }

    /// Get all message IDs sent by a given address.
    pub fn get_sent_messages(env: Env, sender: Address) -> Vec<u64> {
        env.storage()
            .persistent()
            .get(&DataKey::SentMessages(sender))
            .unwrap_or(vec![&env])
    }

    /// Get all message IDs received by a given address.
    pub fn get_received_messages(env: Env, recipient: Address) -> Vec<u64> {
        env.storage()
            .persistent()
            .get(&DataKey::ReceivedMessages(recipient))
            .unwrap_or(vec![&env])
    }

    /// Get statistics for a user.
    pub fn get_user_stats(env: Env, user: Address) -> UserStats {
        env.storage()
            .persistent()
            .get(&DataKey::UserStats(user))
            .unwrap_or(UserStats {
                messages_sent: 0,
                messages_received: 0,
                last_active: 0,
            })
    }

    /// Get the total number of messages stored on-chain.
    pub fn get_message_count(env: Env) -> u64 {
        env.storage().instance().get(&MSG_CTR).unwrap_or(0)
    }

    // ── Admin ────────────────────────────────────────────────────────────

    /// Retrieve the contract admin address.
    pub fn get_admin(env: Env) -> Address {
        env.storage()
            .instance()
            .get(&ADMIN)
            .expect("contract not initialized")
    }
}

// ─── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod test {
    use super::*;
    use soroban_sdk::{testutils::Address as _, Env};

    #[test]
    fn test_initialize() {
        let env = Env::default();
        env.mock_all_auths();
        let contract_id = env.register(QuantumMessageRegistry, ());
        let client = QuantumMessageRegistryClient::new(&env, &contract_id);

        let admin = Address::generate(&env);
        client.initialize(&admin);

        assert_eq!(client.get_admin(), admin);
        assert_eq!(client.get_message_count(), 0);
    }

    #[test]
    fn test_send_and_retrieve_message() {
        let env = Env::default();
        env.mock_all_auths();
        let contract_id = env.register(QuantumMessageRegistry, ());
        let client = QuantumMessageRegistryClient::new(&env, &contract_id);

        let admin = Address::generate(&env);
        let sender = Address::generate(&env);
        let recipient = Address::generate(&env);

        client.initialize(&admin);

        let ciphertext = String::from_str(&env, "U2FsdGVkX1+encrypted_data_here==");
        let content_hash = BytesN::from_array(&env, &[0u8; 32]);

        let msg_id = client.send_message(&sender, &recipient, &ciphertext, &content_hash);
        assert_eq!(msg_id, 1);
        assert_eq!(client.get_message_count(), 1);

        let stored = client.get_message(&msg_id);
        assert_eq!(stored.sender, sender);
        assert_eq!(stored.recipient, recipient);
        assert_eq!(stored.acknowledged, false);
    }

    #[test]
    fn test_acknowledge_message() {
        let env = Env::default();
        env.mock_all_auths();
        let contract_id = env.register(QuantumMessageRegistry, ());
        let client = QuantumMessageRegistryClient::new(&env, &contract_id);

        let admin = Address::generate(&env);
        let sender = Address::generate(&env);
        let recipient = Address::generate(&env);

        client.initialize(&admin);

        let ciphertext = String::from_str(&env, "ciphertext_data");
        let content_hash = BytesN::from_array(&env, &[1u8; 32]);

        let msg_id = client.send_message(&sender, &recipient, &ciphertext, &content_hash);
        client.acknowledge_message(&recipient, &msg_id);

        let stored = client.get_message(&msg_id);
        assert_eq!(stored.acknowledged, true);
    }

    #[test]
    fn test_user_stats() {
        let env = Env::default();
        env.mock_all_auths();
        let contract_id = env.register(QuantumMessageRegistry, ());
        let client = QuantumMessageRegistryClient::new(&env, &contract_id);

        let admin = Address::generate(&env);
        let sender = Address::generate(&env);
        let recipient = Address::generate(&env);

        client.initialize(&admin);

        let ciphertext = String::from_str(&env, "data");
        let hash = BytesN::from_array(&env, &[2u8; 32]);

        client.send_message(&sender, &recipient, &ciphertext, &hash);
        client.send_message(&sender, &recipient, &ciphertext, &hash);

        let sender_stats = client.get_user_stats(&sender);
        assert_eq!(sender_stats.messages_sent, 2);
        assert_eq!(sender_stats.messages_received, 0);

        let recipient_stats = client.get_user_stats(&recipient);
        assert_eq!(recipient_stats.messages_sent, 0);
        assert_eq!(recipient_stats.messages_received, 2);
    }

    #[test]
    fn test_sent_received_tracking() {
        let env = Env::default();
        env.mock_all_auths();
        let contract_id = env.register(QuantumMessageRegistry, ());
        let client = QuantumMessageRegistryClient::new(&env, &contract_id);

        let admin = Address::generate(&env);
        let sender = Address::generate(&env);
        let recipient = Address::generate(&env);

        client.initialize(&admin);

        let ciphertext = String::from_str(&env, "msg");
        let hash = BytesN::from_array(&env, &[3u8; 32]);

        let id1 = client.send_message(&sender, &recipient, &ciphertext, &hash);
        let id2 = client.send_message(&sender, &recipient, &ciphertext, &hash);

        let sent = client.get_sent_messages(&sender);
        assert_eq!(sent.len(), 2);
        assert_eq!(sent.get(0).unwrap(), id1);
        assert_eq!(sent.get(1).unwrap(), id2);

        let received = client.get_received_messages(&recipient);
        assert_eq!(received.len(), 2);
    }
}
