use crate::verkle::proof::Proof;
use block_verkle_proof_extractor::keyvals::KeyVals;

use crate::types::RPCResp;
use reqwest::{
    header::{HeaderMap, HeaderValue, CONTENT_TYPE, USER_AGENT},
    Client,
};
use rlp::{decode, Decodable, DecoderError, Rlp};

use verkle_trie::EdwardsProjective;

use ark_serialize::CanonicalDeserialize;

// This part of code was taken and modified from
// https://github.com/gballet/verkle-block-sample

#[allow(dead_code)]
pub struct VerkleHeader {
    parent_hash: Vec<u8>,
    pub storage_root: Vec<u8>,
    number: Vec<u8>,
    proof: Proof,
    pub keyvals: KeyVals,
}

impl Decodable for VerkleHeader {
    fn decode(rlp: &rlp::Rlp<'_>) -> Result<Self, rlp::DecoderError> {
        Ok(VerkleHeader {
            parent_hash: rlp.at(0)?.as_val::<Vec<u8>>()?,
            storage_root: rlp.at(3)?.as_val::<Vec<u8>>()?,
            number: rlp.at(8)?.as_val::<Vec<u8>>()?,
            proof: rlp.at(16)?.as_val::<Proof>()?,
            keyvals: rlp.at(17)?.as_val::<KeyVals>()?,
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

fn construct_headers() -> HeaderMap {
    let mut headers = HeaderMap::new();
    headers.insert(
        USER_AGENT,
        HeaderValue::from_static("test optimal batch size1"),
    );
    headers.insert(
        CONTENT_TYPE,
        HeaderValue::from_static("application/json; charset=utf-8"),
    );
    headers
}

pub fn decode_block(rlp: String) -> Result<VerkleBlock, anyhow::Error> {
    // rlp string starts with 0x
    // catch panic?
    let rlp_cropped: &str = &rlp[2..];
    let serialized: Vec<u8> = hex::decode(rlp_cropped)?;
    let block: VerkleBlock = decode(&serialized)?;

    Ok(block)
}

#[allow(dead_code)]
pub fn debug_block_info(block: &VerkleBlock) {
    tracing::debug!(
        "Block info:\n- parent hash: {}\n- storage root: {}\n- block number: {}\n",
        hex::encode(block.header.parent_hash.clone()),
        hex::encode(block.header.storage_root.clone()),
        hex::encode(block.header.number.clone())
    );
    let keys = block.header.keyvals.keys.clone();
    tracing::debug!("Key-vals: ");
    for (indx, key) in keys.iter().enumerate() {
        match block.header.keyvals.values[indx] {
            Some(ref val) => tracing::debug!("\t{} => {}", hex::encode(key), hex::encode(val)),
            None => tracing::debug!("\t{} is absent", hex::encode(key)),
        }
    }
}

pub async fn get_rlp(block_number: u64) -> Result<String, reqwest::Error> {
    let client = Client::new();
    let node_end_point: String = "https://rpc.condrieu.ethdevops.io/".to_owned();
    let arg = format!(
        r#"{{"jsonrpc":"2.0","method":"debug_getBlockRlp","params":[{}],"id":"1"}}"#,
        block_number
    );

    let res = client
        .post(&node_end_point)
        .body(arg)
        .headers(construct_headers())
        .send()
        .await?;

    let block_rlp: RPCResp = res.json().await?;

    Ok(block_rlp.result)
}

pub fn verification(
    block: VerkleBlock,
    parent_root: &[u8],
) -> Result<verkle_trie::proof::UpdateHint, anyhow::Error> {
    let root: EdwardsProjective = CanonicalDeserialize::deserialize(parent_root)?;
    let keyvals = block.header.keyvals;

    let (checked, info) = block
        .header
        .proof
        .verkle_proof
        .check(keyvals.keys, keyvals.values, root);

    match checked {
        true => {
            tracing::info!(
                "Good verification of block 0x{}",
                hex::encode(block.header.number)
            );
            match info {
                Some(val) => Ok(val),
                None => Err(anyhow::anyhow!("UpdateHint is none")),
            }
        }
        false => {
            tracing::error!("Bad verification");
            Err(anyhow::anyhow!("Verification didn't work out"))
        }
    }
}
