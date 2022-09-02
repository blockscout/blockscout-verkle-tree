// This part of code was taken and modified from
// https://github.com/gballet/verkle-block-sample

use rlp::{Decodable, DecoderError, Rlp};
use verkle_trie::proof::VerkleProof;

#[allow(dead_code)]
pub(crate) struct Proof {
    pub verkle_proof: VerkleProof,
}

impl Decodable for Proof {
    fn decode(rlp: &Rlp<'_>) -> Result<Self, DecoderError> {
        let serialized_proof = rlp.data()?;
        let proof = VerkleProof::read(serialized_proof).unwrap();

        Ok(Proof {
            verkle_proof: proof,
        })
    }
}
