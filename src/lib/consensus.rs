
use lib::blockchain::{Chain,Blockchain};
use serde_json;
use reqwest::{Client};
use std::io::{Read};

pub struct Consensus;
impl Consensus {

    pub fn resolve_conflicts(blockchain: &mut Blockchain) -> bool {

        let mut new_chain: Option<Chain> = None;
        let mut max_length = blockchain.len();

        let urls: Vec<String> = blockchain
                                    .nodes()
                                    .iter()
                                    .cloned()
                                    .map(|node| node.into_string())
                                    .collect();
        
        let neighbour_chains = Self::get(urls.as_slice());
        for chain in neighbour_chains {
            if chain.len() > max_length && blockchain.valid_chain(&chain) {
                max_length = chain.len();
                new_chain = Some(chain);
            }
        }
        let is_replaced = new_chain.is_some();
        if let Some(longest_chain) = new_chain {
            blockchain.replace(longest_chain);
        }
        is_replaced
    }

    fn get(urls: &[String]) -> Vec<Chain> {
        let chains_raw = Self::get_neighbour_chains(urls);
        Self::deserialize(chains_raw)
    }

    fn get_neighbour_chains(urls: &[String]) -> Vec<String> {
        let mut chains = Vec::<String>::new();
        let client = Client::new();
        //upgrade_todo: rayon or tokio-hyper to request async
        for url in urls {
            let mut res = client.get(url.as_str()).send().expect("todo: handle");
            let mut buffer = String::new();
            res.read_to_string(&mut buffer).expect("todo: handle");
            chains.push(buffer);
        }
        chains
    }

    fn deserialize(chains_raw: Vec<String>) -> Vec<Chain> {
        let mut chains = Vec::<Chain>::new();
        for raw in chains_raw {

            //todo: remove nodes who return invalid chains?
            match serde_json::from_str(raw.as_str()) {
                Ok(chain) => chains.push(chain),
                Err(e) => error!("Unable to deserialize chain {:?} raw: {}", e, raw)
            }            
        }
        chains
    }
}

#[cfg(test)]
mod tests {    
    use lib::consensus::Consensus;
    use env_logger;
    
    #[cfg(feature = "integration")]   
    #[test]
    fn get_neighbour_chains() {
        env_logger::init().unwrap();
        let urls = vec![String::from("http://koalasafe.com")];
        Consensus::get(urls.as_slice());
    }
}
  