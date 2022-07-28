mod proof;
mod keyvals;

use reqwest::Client;
use reqwest::header::{HeaderValue, HeaderMap, CONTENT_TYPE, USER_AGENT};
use rlp::{decode, Decodable, DecoderError, Rlp};
use crate::types::ResponseTest;
use std::{num::ParseIntError};

// https://github.com/gballet/verkle-block-sample
#[allow(dead_code)]
pub struct VerkleHeader {
    parent_hash: Vec<u8>,
    storage_root: Vec<u8>,
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
    header: VerkleHeader,
}

impl Decodable for VerkleBlock {
    fn decode(rlp: &Rlp<'_>) -> Result<Self, DecoderError> {
        let headerrlp = rlp.at(0)?;
        let header: VerkleHeader = VerkleHeader::decode(&headerrlp)?;
        Ok(VerkleBlock { header })
    }
}

// with 0x
pub fn decode_hex(s: &str) -> Result<Vec<u8>, ParseIntError> {
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
    let serialized: Vec<u8> = decode_hex(&block_rlp.result).unwrap();
    let block: VerkleBlock = decode(&serialized).expect("could not decode verkle block");
    log::info!(
        "Block info:\n- parent hash: {}\n- storage root: {}\n- block number: {}\n- rlp lenght: {}",
        hex::encode(block.header.parent_hash), hex::encode(block.header.storage_root), hex::encode(block.header.number), serialized.len()
    );
    // let keyvals = block.header.keyvals;
    // for (k, v) in keyvals.keys.iter().zip(keyvals.values.clone()) {
    //     match v {
    //         Some(ref val) => log::info!("\t{} => {}", hex::encode(k), hex::encode(val)),
    //         None => log::info!("\t{} is absent", hex::encode(k)),
    //     }
    // }

    Ok(format!("{}", serialized.len()))
}
