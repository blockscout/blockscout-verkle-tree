use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, PartialEq)]
pub struct VerkleReq {
    pub block_number: String
}

#[derive(Debug, Serialize)]
pub struct VerkleResp {
    pub block_number: String
}
