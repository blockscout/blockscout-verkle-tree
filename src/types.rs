use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, PartialEq)]
pub struct VerkleReq {
    pub block_number: u64
}

#[derive(Debug, Serialize)]
pub struct VerkleResp {#[allow(dead_code)]
    pub block_rlp: String
}

#[allow(dead_code)]
#[derive(Deserialize, Debug)]
pub struct RPCResp {
    jsonrpc: String,
    #[serde(alias = "result", alias = "error")]
    pub result: String,
    id: String,
}
