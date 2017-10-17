
use chrono;

use lib::hasher::*;
use lib::transaction::Transaction;
use std::collections::BTreeSet;
use std::collections::HashSet;
use self::chrono::offset::Utc;
use url::{Url};

pub type Chain = BTreeSet<Block>;

#[derive(Debug)]
pub struct Blockchain {
    chain: Chain,
    //not a lot of sorted options in stdlib...
    current_transactions: BTreeSet<Transaction>,
    nodes: HashSet<Url>,
    difficulty: u64
}

#[derive(Debug)]
#[derive(Serialize, Deserialize)]
#[derive(PartialEq, Eq, PartialOrd, Ord)]
pub struct Block {
    pub index: usize,
    pub timestamp: i64,
    pub proof: u64,
    pub previous_hash: String,
    pub transactions: BTreeSet<Transaction>
}

impl Blockchain {

    #[cfg(test)]
    pub fn new() -> Blockchain {
        Self::new_with(3)
    }
    pub fn new_with(difficulty: u64) -> Blockchain {
        let mut blockchain = Blockchain {
            chain: BTreeSet::new(),
            current_transactions: BTreeSet::new(),
            nodes: HashSet::new(),
            difficulty: difficulty
        };
        blockchain.new_block(100, String::from("Genesis block."));
        blockchain
    }
    
    ///
    /// Add a new transaction
    /// 
    /// returns: the index of the block it will be added to
    pub fn new_transaction(&mut self, transaction: Transaction) -> usize {        
        self.current_transactions.insert(transaction);
        //It will be added to the index of the next block
        self.last_block().index + 1
    }

    ///
    /// Mine a new block
    /// 
   pub fn mine(&mut self) -> &Block {
        // We run the proof of work algorithm to get the next proof...    
        let new_block_proof = self.new_block_proof();
        //Got it. Give ourselves the new coin (block?)
        //The sender is "0" to signify that this node has mined a new coin.
        self.new_transaction(Transaction::new("0".into(), "my node address".into(), 1));
        let previous_hash = self.hash_last_block();
        //Forge the new Block by adding it to the chain
        let mined_block = self.new_block(new_block_proof, previous_hash);
        &mined_block
    }

    pub fn chain(&self) -> &BTreeSet<Block> {
        &self.chain
    }

    ///
    /// Add a new node
    /// 
    pub fn register_node(&mut self, address: Url) -> bool {
        self.nodes.insert(address)
    }

    pub fn nodes(&self) -> &HashSet<Url> {
        &self.nodes
    }

    pub fn replace(&mut self, new_chain: Chain) {
        self.chain = new_chain;
    }

    pub fn len(&self) -> usize {
        self.chain.len()
    }

    fn create_block(&mut self, proof: u64, previous_hash: String) -> Block {
        //Current transactions get moved to this block and are cleared to start
        //collecting the next block's transactions
        let mut txns = BTreeSet::new();
        txns.append(&mut self.current_transactions);
        Block {
            index: self.chain.len() + 1,
            timestamp: Utc::now().timestamp(),
            proof: proof,
            previous_hash: previous_hash,
            transactions: txns
        }
    }
    
    ///
    ///Create a new Block 
    ///
    fn new_block(&mut self, proof: u64, previous_hash: String) -> &Block {
        let block = self.create_block(proof, previous_hash);
        self.chain.insert(block);
        &self.chain.iter().next_back().expect("Just added element")
    }
  
    fn last_block(&self) -> &Block {
        //it's a double-ended iterator, and it's sorted, so it should be fast
        self.chain.iter().next_back().expect("Chain empty. Expected genesis block")
    }

    fn hash(block: &Block) -> Result<String, String> {
       self::hash(block)
    }

    ///
    ///Simple Proof of Work Algorithm:
    ///          - Find a number p' such that hash(pp') contains leading 4 zeroes, where p is the previous p'
    ///          - p is the previous proof, and p' is the new proof
    fn proof_of_work(last_proof: u64, difficulty: u64) -> u64 {
        info!("Mining from last_proof {}...", last_proof);
        let mut proof = 0;
        while !Self::valid_proof(last_proof, proof, difficulty) {
             proof += 1
        }
        debug!("Took {} iterations",proof);
        return proof
    }

    /// Validates the Proof
    /// i.e. does the hash of last_proof and this proof start with 000?
    fn valid_proof(last_proof: u64, proof: u64, difficulty: u64) -> bool {
        
        //todo: don't recalculate every time
        let hash_prefix = "0".repeat(difficulty as usize); //"000"

        let guess = format!("{}{}", last_proof, proof);
        let guess_hash =  self::hash_string(guess);
        let is_valid = guess_hash.starts_with(hash_prefix.as_str());
        if is_valid {
            debug!("guess_hash: {}", guess_hash);
        }
        is_valid
    }

    fn new_block_proof(&self) -> u64{
        let last_block = self.last_block();
        let last_proof = last_block.proof;
        //Mine it!
        Self::proof_of_work(last_proof, self.difficulty)
    }

    fn hash_last_block(&self) -> String {
        let last_block = self.last_block();
        //TODO: Don't panic here
        Self::hash(last_block).expect("hash block failed")
    }

    ///
    ///         Determine if a given blockchain is valid
    /// 
    pub fn valid_chain(&self, chain: &Chain) -> bool {        
        debug!("{} blocks in chain.", chain.len());
        let mut previous_block_opt: Option<&Block> = None;        
        for block in chain {
            if let Some(previous_block) = previous_block_opt {
                //Check the hash and proof
                if !Self::check_hash(previous_block, block) || !Self::check_proof(previous_block, block, self.difficulty) {
                    return false;
                }               
            }
            previous_block_opt = Some(&block);
        }
        true
    }

    fn check_hash(previous_block: &Block, current_block: &Block) -> bool {
        let previous_block_hash = Self::hash(previous_block).expect("//todo handle hash failure");
        if current_block.previous_hash != previous_block_hash {
            warn!("HASH MISMATCH {} <> {}", current_block.previous_hash, previous_block_hash);
            return false
        }
        true
    }

    fn check_proof(previous_block: &Block, current_block: &Block, difficulty: u64) -> bool {
        if !Self::valid_proof(previous_block.proof, current_block.proof, difficulty) {                
            warn!("PROOF MISMATCH {} <> {}", previous_block.proof, current_block.proof);
            return false
        }
        true
    }
}

#[cfg(test)]
mod tests {
    use lib::blockchain::Blockchain;
    use lib::transaction::Transaction;
    use url::Url;

    #[test]
    fn new_transaction() {
        let mut blockchain = Blockchain::new();
        let txn = Transaction::new(String::from("a"), String::from("b"), 100);
        let _idx = blockchain.new_transaction(txn);
        let last_txn = blockchain.current_transactions.iter().next_back().expect("expected a txn");
        assert_eq!(last_txn.sender, String::from("a"));
        assert_eq!(last_txn.recipient, String::from("b"));
        assert_eq!(last_txn.amount, 100);
    }

     #[test]
    fn new_block() {
        let mut blockchain = Blockchain::new();
        let txn = Transaction::new(String::from("a"), String::from("b"), 100);
        blockchain.new_transaction(txn);
        
        let a = blockchain.current_transactions.len();
        assert_eq!(1, a , "1 transaction");
    
        blockchain.new_block(2, String::from("abc"));
                 
        let b = blockchain.current_transactions.len();
        assert_eq!(0, b, "New block should clear transactions (which were on the previous block");    
    }
    
    #[test]
    fn hash() {
        let mut blockchain = Blockchain::new();       
        blockchain.new_block(2, String::from("abc"));
        let block = blockchain.last_block();
        let hash = Blockchain::hash(block);
        let hash2 = Blockchain::hash(block);
        println!("{:?}", hash);
        assert!(hash.is_ok());
        assert_eq!(hash.unwrap(), hash2.unwrap(), "Expected same block to hash to the same value");
        //assert!(hash.unwrap().len() > 10, "expected a longer hash");       
    }

    #[test]
    fn valid_proof_false() {
        assert_eq!(Blockchain::valid_proof(100,1, 3), false);
    }
    
    #[cfg(feature = "mining-tests")]    
    #[test]
    fn proof_of_work() {
        let blockchain = Blockchain::new();     
        println!("Starting proof of work... (long running)");
        let proof = blockchain.new_block_proof();
        println!("Finished proof of work: {}", proof);
        assert!(proof > 1, "expected a higher proof");
        assert!(Blockchain::valid_proof(100, proof));
    }

    #[test]
    fn chain() {
        let mut blockchain = Blockchain::new();     
        
        assert_eq!(blockchain.chain().len(),  1, "Expected 1 block (genesis)");
        blockchain.new_block(100, "abc".into());
        assert_eq!(blockchain.chain().len(),  2, "Expected 2 blocks");
    }

    #[test]
    fn register_node() {
        let mut blockchain = Blockchain::new();     
        let test_local_url = Url::parse("http://localhost:9000").expect("valid url");
        blockchain.register_node(test_local_url.clone());
        assert_eq!(blockchain.nodes().len(),  1, "Expected 1 node");
        blockchain.register_node(test_local_url);
        assert_eq!(blockchain.nodes().len(),  1, "Expected 1 node after dupe add (idempotent)");
    }

    #[test]
    fn valid_chain_invalid_hash() {
        //env_logger::init().unwrap();
        let mut blockchain = Blockchain::new();
        let txn = Transaction::new(String::from("a"), String::from("b"), 100);
        blockchain.new_transaction(txn);
        //invalid hash
        blockchain.new_block(2, String::from("abc"));
        assert!(!blockchain.valid_chain(&blockchain.chain), "blockchain not valid (hash mismatch)");
    }


    #[test]
    fn valid_chain_invalid_proof() {
        let mut blockchain = Blockchain::new();
        let txn = Transaction::new(String::from("a"), String::from("b"), 100);
        blockchain.new_transaction(txn);
        //valid hash, invalid proof
        let hash = blockchain.hash_last_block();
        blockchain.new_block(2, hash);

        assert!(!blockchain.valid_chain(&blockchain.chain), "blockchain not valid (proof mismatch)");
    }

    #[test]
    #[cfg(feature = "mining-tests")]    
    fn valid_chain_ok() {
        let mut blockchain = Blockchain::new();
        let txn = Transaction::new(String::from("a"), String::from("b"), 100);
        blockchain.new_transaction(txn);
        //valid hash, invalid proof
        blockchain.mine();
        assert!(blockchain.valid_chain(&blockchain.chain), "blockchain should be valid with a mined block");
    }
    
}

// import hashlib
// import json
// from time import time
// from urllib.parse import urlparse
// from uuid import uuid4

// import requests
// from flask import Flask, jsonify, request


// class Blockchain(object):
//     def __init__(self):
//         self.current_transactions = []
//         self.chain = []
//         self.nodes = set()

//         # Create the genesis block
//         self.new_block(previous_hash=1, proof=100)

//     def register_node(self, address):
//         """
//         Add a new node to the list of nodes

//         :param address: <str> Address of node. Eg. 'http://192.168.0.5:5000'
//         :return: None
//         """

//         parsed_url = urlparse(address)
//         self.nodes.add(parsed_url.netloc)

//     def valid_chain(self, chain):
//         """
//         Determine if a given blockchain is valid

//         :param chain: <list> A blockchain
//         :return: <bool> True if valid, False if not
//         """

//         last_block = chain[0]
//         current_index = 1

//         while current_index < len(chain):
//             block = chain[current_index]
//             print(f'{last_block}')
//             print(f'{block}')
//             print("\n-----------\n")
//             # Check that the hash of the block is correct
//             if block['previous_hash'] != self.hash(last_block):
//                 return False

//             # Check that the Proof of Work is correct
//             if not self.valid_proof(last_block['proof'], block['proof']):
//                 return False

//             last_block = block
//             current_index += 1

//         return True

//     def resolve_conflicts(self):
//         """
//         This is our consensus algorithm, it resolves conflicts
//         by replacing our chain with the longest one in the network.

//         :return: <bool> True if our chain was replaced, False if not
//         """

//         neighbours = self.nodes
//         new_chain = None

//         # We're only looking for chains longer than ours
//         max_length = len(self.chain)

//         # Grab and verify the chains from all the nodes in our network
//         for node in neighbours:
//             response = requests.get(f'http://{node}/chain')

//             if response.status_code == 200:
//                 length = response.json()['length']
//                 chain = response.json()['chain']

//                 # Check if the length is longer and the chain is valid
//                 if length > max_length and self.valid_chain(chain):
//                     max_length = length
//                     new_chain = chain

//         # Replace our chain if we discovered a new, valid chain longer than ours
//         if new_chain:
//             self.chain = new_chain
//             return True

//         return False

//     def new_block(self, proof, previous_hash=None):
//         """
//         Create a new Block in the Blockchain

//         :param proof: <int> The proof given by the Proof of Work algorithm
//         :param previous_hash: (Optional) <str> Hash of previous Block
//         :return: <dict> New Block
//         """

//         block = {
//             'index': len(self.chain) + 1,
//             'timestamp': time(),
//             'transactions': self.current_transactions,
//             'proof': proof,
//             'previous_hash': previous_hash or self.hash(self.chain[-1]),
//         }

//         # Reset the current list of transactions
//         self.current_transactions = []

//         self.chain.append(block)
//         return block

//     def new_transaction(self, sender, recipient, amount):
//         """
//         Creates a new transaction to go into the next mined Block

//         :param sender: <str> Address of the Sender
//         :param recipient: <str> Address of the Recipient
//         :param amount: <int> Amount
//         :return: <int> The index of the Block that will hold this transaction
//         """
//         self.current_transactions.append({
//             'sender': sender,
//             'recipient': recipient,
//             'amount': amount,
//         })

//         return self.last_block['index'] + 1

//     @property
//     def last_block(self):
//         return self.chain[-1]

//     @staticmethod
//     def hash(block):
//         """
//         Creates a SHA-256 hash of a Block

//         :param block: <dict> Block
//         :return: <str>
//         """

//         # We must make sure that the Dictionary is Ordered, or we'll have inconsistent hashes
//         block_string = json.dumps(block, sort_keys=True).encode()
//         return hashlib.sha256(block_string).hexdigest()

//     def proof_of_work(self, last_proof):
//         """
//         Simple Proof of Work Algorithm:
//          - Find a number p' such that hash(pp') contains leading 4 zeroes, where p is the previous p'
//          - p is the previous proof, and p' is the new proof

//         :param last_proof: <int>
//         :return: <int>
//         """

//         proof = 0
//         while self.valid_proof(last_proof, proof) is False:
//             proof += 1

//         return proof

//     @staticmethod
//     def valid_proof(last_proof, proof):
//         """
//         Validates the Proof

//         :param last_proof: <int> Previous Proof
//         :param proof: <int> Current Proof
//         :return: <bool> True if correct, False if not.
//         """

//         guess = f'{last_proof}{proof}'.encode()
//         guess_hash = hashlib.sha256(guess).hexdigest()
//         return guess_hash[:4] == "0000"


// # Instantiate the Node
// app = Flask(__name__)

// # Generate a globally unique address for this node
// node_identifier = str(uuid4()).replace('-', '')

// # Instantiate the Blockchain
// blockchain = Blockchain()


// @app.route('/mine', methods=['GET'])
// def mine():
//     # We run the proof of work algorithm to get the next proof...
//     last_block = blockchain.last_block
//     last_proof = last_block['proof']
//     proof = blockchain.proof_of_work(last_proof)

//     # We must receive a reward for finding the proof.
//     # The sender is "0" to signify that this node has mined a new coin.
//     blockchain.new_transaction(
//         sender="0",
//         recipient=node_identifier,
//         amount=1,
//     )

//     # Forge the new Block by adding it to the chain
//     block = blockchain.new_block(proof)

//     response = {
//         'message': "New Block Forged",
//         'index': block['index'],
//         'transactions': block['transactions'],
//         'proof': block['proof'],
//         'previous_hash': block['previous_hash'],
//     }
//     return jsonify(response), 200


// @app.route('/transactions/new', methods=['POST'])
// def new_transaction():
//     values = request.get_json()

//     # Check that the required fields are in the POST'ed data
//     required = ['sender', 'recipient', 'amount']
//     if not all(k in values for k in required):
//         return 'Missing values', 400

//     # Create a new Transaction
//     index = blockchain.new_transaction(values['sender'], values['recipient'], values['amount'])

//     response = {'message': f'Transaction will be added to Block {index}'}
//     return jsonify(response), 201


// @app.route('/chain', methods=['GET'])
// def full_chain():
//     response = {
//         'chain': blockchain.chain,
//         'length': len(blockchain.chain),
//     }
//     return jsonify(response), 200


// @app.route('/nodes/register', methods=['POST'])
// def register_nodes():
//     values = request.get_json()

//     nodes = values.get('nodes')
//     if nodes is None:
//         return "Error: Please supply a valid list of nodes", 400

//     for node in nodes:
//         blockchain.register_node(node)

//     response = {
//         'message': 'New nodes have been added',
//         'total_nodes': list(blockchain.nodes),
//     }
//     return jsonify(response), 201


// @app.route('/nodes/resolve', methods=['GET'])
// def consensus():
//     replaced = blockchain.resolve_conflicts()

//     if replaced:
//         response = {
//             'message': 'Our chain was replaced',
//             'new_chain': blockchain.chain
//         }
//     else:
//         response = {
//             'message': 'Our chain is authoritative',
//             'chain': blockchain.chain
//         }

//     return jsonify(response), 200


// if __name__ == '__main__':
//     app.run(host='0.0.0.0', port=5000)