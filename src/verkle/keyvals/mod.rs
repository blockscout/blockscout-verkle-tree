// https://github.com/gballet/verkle-block-sample

use rlp::{Decodable, DecoderError, Rlp};
use std::convert::TryInto;

#[derive(Debug)]
pub struct Tuple(Vec<u8>, Vec<u8>);

impl Decodable for Tuple {
    fn decode(rlp: &Rlp<'_>) -> std::result::Result<Self, DecoderError> {
        Ok(Tuple(rlp.val_at::<Vec<u8>>(0)?, rlp.val_at::<Vec<u8>>(1)?))
    }
}

impl TryInto<([u8; 32], Option<[u8; 32]>)> for Tuple {
    type Error = String;
    fn try_into(
        self,
    ) -> std::result::Result<
        ([u8; 32], Option<[u8; 32]>),
        <Self as TryInto<([u8; 32], Option<[u8; 32]>)>>::Error,
    > {
        let mut second = None;

        if !self.1.is_empty() {
            second = Some(self.1.try_into().unwrap());
        }

        Ok((self.0.try_into().unwrap(), second))
    }
}

pub struct KeyVals {
    pub keys: Vec<[u8; 32]>,
    pub values: Vec<Option<[u8; 32]>>,
}
 
impl Decodable for KeyVals {
    fn decode(rlp: &Rlp<'_>) -> Result<Self, DecoderError> {
        let (keys, values): (Vec<[u8; 32]>, Vec<Option<[u8; 32]>>) = rlp
            .iter()
            .map(|r| r.as_val::<Tuple>().unwrap().try_into().unwrap())
            .unzip();

        Ok(KeyVals {
            keys,
            values,
        })
    }
}
