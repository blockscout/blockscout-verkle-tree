mod proof;
mod keyvals;

use reqwest::Client;
use reqwest::header::{HeaderValue, HeaderMap, CONTENT_TYPE, USER_AGENT};
use rlp::{decode, Decodable, DecoderError, Rlp};
use crate::types::ResponseTest;
use std::{num::ParseIntError};

use verkle_trie::EdwardsProjective;

use ark_serialize::CanonicalDeserialize;

// https://github.com/gballet/verkle-block-sample
#[allow(dead_code)]
pub struct VerkleHeader {
    parent_hash: Vec<u8>,
    pub storage_root: Vec<u8>,
    number: Vec<u8>,
    proof: proof::Proof,
    keyvals: keyvals::KeyVals,
}

impl Decodable for VerkleHeader {
    fn decode(rlp: &rlp::Rlp<'_>) -> Result<Self, rlp::DecoderError> {
        Ok(VerkleHeader {
            parent_hash: rlp.at(0)?.as_val::<Vec<u8>>()?,
            storage_root: rlp.at(3)?.as_val::<Vec<u8>>()?,
            number: rlp.at(8)?.as_val::<Vec<u8>>()?,
            proof: rlp.at(16)?.as_val::<proof::Proof>()?,
            keyvals: rlp.at(17)?.as_val::<keyvals::KeyVals>()?,
        })
    }
}

pub struct VerkleBlock {
    pub header: VerkleHeader,
}

impl Decodable for VerkleBlock {
    fn decode(rlp: &Rlp<'_>) -> Result<Self, DecoderError> {
        let headerrlp = rlp.at(0)?;
        let header: VerkleHeader = VerkleHeader::decode(&headerrlp)?;
        Ok(VerkleBlock { header })
    }
}

// with 0x
fn decode_hex(s: &str) -> Result<Vec<u8>, ParseIntError> {
    (2..s.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&s[i..i + 2], 16))
        .collect()
}

fn construct_headers() -> HeaderMap {
    let mut headers = HeaderMap::new();
    headers.insert(USER_AGENT, HeaderValue::from_static("test optimal batch size1"));
    headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json; charset=utf-8"));
    headers
}

// ! rlp string starts with 0x
pub fn decode_block(rlp: String) -> Result<VerkleBlock, DecoderError> {
    let serialized: Vec<u8> = decode_hex(&rlp).unwrap();
    let block: VerkleBlock = decode(&serialized)?;

    Ok(block)
}

pub fn print_block_info(block: &VerkleBlock) {
    println!(
            "Block info:\n- parent hash: {}\n- storage root: {}\n- block number: {}\n",
            hex::encode(block.header.parent_hash.clone()),
            hex::encode(block.header.storage_root.clone()),
            hex::encode(block.header.number.clone())
        );
    let keys = block.header.keyvals.keys.clone();
    println!("Kkey-vals: ");
    for (indx, key) in keys.iter().enumerate() {
        match block.header.keyvals.values[indx] {
            Some(ref val) => println!("\t{} => {}", hex::encode(key), hex::encode(val)),
            None => println!("\t{} is absent", hex::encode(key)),
        }
    }
}

pub async fn get_rlp(block_number: u64) -> Result<String, reqwest::Error> {
    let client = Client::new();
    let node_end_point: String = "https://rpc.condrieu.ethdevops.io/".to_owned();
    let arg = format!(r#"{{"jsonrpc":"2.0","method":"debug_getBlockRlp","params":[{}],"id":"1"}}"#, block_number);

    let res = client.post(&node_end_point)
        .body(arg)
        .headers(construct_headers())
        .send()
        .await?;

    let block_rlp: ResponseTest = res.json().await?;

    Ok(block_rlp.result)
}

pub fn verification(block: VerkleBlock, parent_root: String) -> Result<verkle_trie::proof::UpdateHint, anyhow::Error> {
    let parent_root = hex::decode(parent_root).unwrap();
    let root: EdwardsProjective = CanonicalDeserialize::deserialize(&parent_root[..]).unwrap();
    let keyvals = block.header.keyvals;

    let (checked, info) = block
        .header
        .proof
        .verkle_proof
        .check(keyvals.keys, keyvals.values, root);

    match checked {
        true => {
            println!("Good verification");
            Ok(info.unwrap())
        },
        false => {
            println!("Bad verification");
            Err(anyhow::anyhow!("Verification didn't work out"))
        }
    }
}

#[allow(dead_code)]
pub async fn save_rlp(rlp: String, num: u64) -> Result<(), std::io::Error> {
    use std::fs::File;
    use std::io::prelude::*;

    let mut file = File::create(format!("condrieu.block{}.rlp", num))?;
    file.write_all(&decode_hex(&rlp).unwrap())?;

    Ok(())
}


// Usefull, but !excess!

// fn build_tree(keys, values) {
    // let db = MemoryDb::new();
    // let mut trie = Trie::new(TestConfig::new(db));

    // for (idx, key) in block.header.keyvals.keys.iter().enumerate() {
    //     trie.insert_single(key.clone(), block.header.keyvals.values[idx].unwrap());
    // }

    // let root_commitment = trie.root_commitment();
    // let root_hash = trie.root_hash();
    // println!("\nroot hash = {:?}", hex::encode(scalar_to_array(&root_hash)));

    // println!("\nroot comm = {:?}\n", root_commitment);

    // println!("\ntrie = {:?}\n", trie);
// }
    // use verkle_trie::database::ReadOnlyHigherDb;

    // println!("get branch meta");

    // let root = vec![];
    // // // rec(&root, &trie);

    // let branch_meta = trie.storage.get_branch_meta(&root);

    // println!("{:?}", branch_meta);

    // // // println!("children");
    // let children = trie.storage.get_branch_children(&root);

    // super! how to get_branch_children with that data (recursion)???
    // println!("{:?}", children);

    // for child in children {
    //     println!("\nnew child");
    //     println!("prefix (in decimal) : {:#02x}", child.0);

    //     match child.1 {
    //         BranchChild::Stem(stem_id) => println!("stem-data : {:?}", trie.storage.get_stem_meta(stem_id)),
    //         BranchChild::Branch(b_meta) => println!("meta : {:?}", b_meta),
    //     };
    // }
    // for (idx, key) in block.header.keyvals.keys.iter().enumerate() {
    

    //     let stem_meta = trie.storage.get_stem_meta(*key);

    //     println!("stem_meta:");
    //     println!("{:?}", encode_hex(stem_meta));
    //     let leaf = trie.storage.get_leaf(*key).unwrap_or_default();
    //     println!("{:?}", encode_hex(&leaf));

    //     println!("vs");
    //     println!("{:?}", encode_hex(&block.header.keyvals.values[idx].unwrap_or_default()));
    //     db.get_leaf(key);
    // }
    
    // println!("\ndb = {:?}", db);

    // println!("\nproof = {:?}", block.header.proof.verkle_proof);
    // let root = vec![];
    // let meta = trie.storage.get_branch_meta(&root).unwrap();
    // let mut file = Vec::<u8>::new();
    // let res = block.header.proof.verkle_proof.write(&mut file);
    // match res {
    //     Ok(()) => println!("{:?}", file),
    //     Err(e) => println!("fuck = {:?}", e)
    // }
    
    // for 02
    // let parent_root_string = "ed1b129ad7a66818bb92034770639cb2159ea570c2355811ff428b32f7870d33";
    
    // let parent_root_string = "4ab9e47a3b8508c264b56bca01a82ff0fa01dddb8e97d012058c49e133850539";

    // let parent_root = hex::decode(parent_root_string).unwrap();
    // let root: EdwardsProjective = CanonicalDeserialize::deserialize(&parent_root[..]).unwrap();

    // // keyvals!
    // // println!("\n");
    // // let mut keys = block.header.keyvals.keys.clone();
    // // let mut values = block.header.keyvals.values.clone();
    // let keyvals = block.header.keyvals;
    // for (k, v) in keyvals.keys.iter().zip(keyvals.values.clone()) {
    //     match v {
    //         Some(ref val) => println!("\t{} => {}", hex::encode(k), hex::encode(val)),
    //         None => println!("\t{} is absent", hex::encode(k)),
    //     }
    // }
    // no way get full info from header, we need to build tree :(
    // println!("{:?}", block.header.proof.verkle_proof.verification_hint);

    // fuck there's a lot of data...
    // println!("\n\ncomms sorted by path");
    // for comm in block.header.proof.verkle_proof.comms_sorted {
    //     let bytes = comm.to_bytes();
    //     println!("{:?}", encode_hex(&bytes));
    // }

    // println!("\n\n");
    // let keyvals = block.header.keyvals;

    // println!("key len {:?}\nvalue len {:?}", keyvals.keys.len(), keyvals.values.len());
    // let (checked, info) = block
    //     .header
    //     .proof
    //     .verkle_proof
    //     .check(keyvals.keys, keyvals.values, root);


    // println!("checked: {}", checked);

    // let root = vec![];
    // let meta = trie.storage.get_branch_meta(&root).unwrap();

    // println!("{:?}", meta.commitment);
    // let proof = verkle_trie::proof::prover::create_verkle_proof(&trie.storage, keyvals.keys.clone());
    // let values: Vec<_> = keyvals.keys.iter().map(|val| Some(*val)).collect();
    // let (ok, _) = proof.check(keyvals.keys, values, meta.commitment);

    // use verkle_trie::database::ReadOnlyHigherDb;

    // let db = MemoryDb::new();
    // let mut trie = Trie::new(TestConfig::new(db));

    // let mut keys = Vec::new();
    // let mut values = Vec::new();
    // for (indx, key) in keyvals.keys.iter().enumerate() {
    //     // println!("key = {:?}, value = {:?}", hex::decode(*key), keyvals.values[indx]);
    //     match keyvals.values[indx] {
    //         Some(v) => {
    //             trie.insert_single(*key, v);
    //             keys.push(*key);
    //             values.push(v);
    //         },
    //         None => {println!("skip")}
    //     }
    // }
    
    // let meta = trie.storage.get_branch_meta(&root).unwrap();

    // println!("{:?}", data);
// fn rec(node: &[u8], trie: &Trie<verkle_trie::database::memory_db::MemoryDb, verkle_trie::committer::test::TestCommitter>) {
//     use verkle_trie::database::ReadOnlyHigherDb;
//     let branch_meta = trie.storage.get_branch_meta(&node);
//     match branch_meta {
//         Some(meta) => {
//             println!("{:?}", meta);
//         },
//         None => return
//     }
// }
// fn scalar_to_array(scalar: &Fr) -> [u8; 32] {
//     let mut bytes = [0u8; 32];
//     scalar.serialize_uncompressed(&mut bytes[..]).unwrap();
//     bytes
// }

