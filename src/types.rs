use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, PartialEq, Eq)]
pub struct VerkleReq {
    pub block_number: u64,
}

#[derive(Debug, Serialize)]
pub struct VerkleResp {
    #[allow(dead_code)]
    // pub image: Vec<u8>
    pub image: String,
}

#[allow(dead_code)]
#[derive(Deserialize, Debug)]
pub struct RPCResp {
    jsonrpc: String,
    #[serde(alias = "result", alias = "error")]
    pub result: String,
    id: String,
}
