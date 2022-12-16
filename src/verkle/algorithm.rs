use crate::verkle::proof::Proof;
use block_verkle_proof_extractor::keyvals::KeyVals;

use crate::types::RPCResp;
use reqwest::{
    header::{HeaderMap, HeaderValue, CONTENT_TYPE, USER_AGENT},
    Client,
};
use rlp::{decode, Decodable, DecoderError, Rlp};

use verkle_trie::Element;

use ark_serialize::CanonicalDeserialize;

// This part of code was taken and modified from
// https://github.com/gballet/verkle-block-sample

#[allow(dead_code)]
pub struct VerkleHeader {
    pub parent_hash: Vec<u8>,
    pub storage_root: Vec<u8>,
    pub number: Vec<u8>,
    proof: Proof,
    pub keyvals: KeyVals,
}

impl Decodable for VerkleHeader {
    fn  decode(rlp: &rlp::Rlp<'_>) -> Result<Self, rlp::DecoderError> {
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
    let root: Element = CanonicalDeserialize::deserialize(parent_root)?;
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decode_proof() {
        let proof_string = "b902d20000000006000000080a0a0808080400000055fc28df8d54aef6ffb43695ef3fed55993ae008dc033c36bb48d3efa131e7d85c3e125f152e0b296a8fc6a7506c52ea7cf364a85d2f38425c495845b8e5fcaa621c65eed175759bed2c1293ba153fcbcc03a77ec5b8be6a70525284fadfe5ce2c40ed9e5cc59ec79c1f64558e7712705119f5b6adba6abe67b0a85eea6a57a372012d816767142d06b30a45da766d3453d5eea9f411e6ee25ab672da1580d09606162ddc947c873020f4e251c671b680146bffe0ce1a26d6a426f00a3206a7a59be7ae8bf499671db45344db4de2d610c2e7788dadf3c3798ffdaba155028566f1499f496b9fd0507954835c425270f3b08c68ab5475b7fea7dcb4de1fabb9b2c7ef6dcb3858f66ef02925afb5fddc6be130d470a7613058d3bf176cb5fa53d1b778f4f1f4b3b176af9fd234be9844f065a650a3facc3f18550948ae0eb8cb92a48a0a0413bd051054e27e359834a584c63da39cc094deac2c2290f60c6066b400efec9f3cbb4428d5972c2829a1dbaab6363507e1b4391210e26d3a03ada6e3384cf4db8a53f35bc30fbb1015104658e94160f29ac7becd4e75bfa8e46864216069023ab2a2c12a2be1c8ceed81c05645b3c6f9023339af49e62626e5b890c710c6d4c15c0b0fc72bcf3e609c3687b85ebde41101b1a7487d64a0edaf56bb9125ab05310b17001004453ad8e7ec56a804a97f9f39218488fbdcb6d90a7a95f1a965420c7cda79b8436cac411c00996466b19cb4591f17bba2014d0d7de150e6042fec3a7a55f4cc4114b8b85322d6694abbedd1c2acad8a2a41908f60d9ee30ed851b6a47e5ad2fe9aa22d5c26e11c2e4e04a76f8ebb6d8a7b7e265e9a032463508bbf4f7304b5d353b65bb49b22abbc3316675c793171e3d5234a5f71526d32e70de0056d4c9225dc65c35adbc36c2eb184db805662cb10e9ecb34553b909b4ac2e49eb5558dcf2d6fe86a0bff9d7d21f57c78bd2ed82e125e78f2c2ea91b";
        let proof_raw = hex::decode(proof_string).expect("proof string decoding");

        let proof: Proof = decode(&proof_raw).expect("decoding");
    }

    #[test]
    fn test_decode_block() {
        let block_string = "0xf905bbf905b6a0bdab9f2bea8cfa999926784537c4bf8406b0a6172586ae475c5df70620d9a1dfa01dcc4de8dec75d7aab85b567b6ccd41ad312451b948a7413f0a142fd40d4934794f97e180c050e5ab072211ad2c213eb5aee4df134a0541a5fc884be032dcd0cfed8c273b02cf79b1910a0fba04214d889feacba0adca056e81f171bcc55a6ff8345e692c0f86e5b48e01b996cadc001622fb5e363b421a056e81f171bcc55a6ff8345e692c0f86e5b48e01b996cadc001622fb5e363b421b90100000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000008302431382012c83403cd08084633bfbdc99d883010a15846765746888676f312e31382e36856c696e7578a0480adda6439ccea6fb3651c12b095a7c74da4b8405f086e9393d0a9acb8dccda884ecf6149b0f26cb307b9028d00000000010000000a020000003724b0b0ee9613839978c8b72fddfa3dd706a517f629bccf0d043702a1ea119a6fdedbe27108a19f47c427f3305e560435143eac82ee18cef8f34dc051f8ff033050362f80552c21ad2ad5bdf534094a0d18ee126e214ef06e7c97db9cc893664ea0090c8960297b1d466eb739571152937f70a6b13294a0239651bff2e024fe0be1cf53ce11fb0b50c10552dd2ecdf40f0fb88e9e8683d7424923fac142afd01bff4f22cdf950b98ada2567cf9462de8e8046777b8e673f13d784b2c84196bd68a757dcf9f0804d18b38db1bdb480bb4ae07d441f5e2d17a193aeff76c40b8b730bb186355203b3f1866ac5fbd86b1ba03bc5054034dbbd870e0b3c267c0385524c2fad495ea6f5f0d4de7790af23023f09fdd61e5e939a74694965cd81b61f1bcfa08468ff3468c3e738fef5e96760e65402e5641587cde3bfb6f769eb7f7a1047f4ff3d95c87ff2e5fc91d1c2493a659cca25669b2e18bbac936a2ea95de61991b57c7901a4443e95e7ced8006597645f52b5b8c576ccdf36232e5e72434f3e5f28f46f8514f0ca33cfb235db1fcb3c62ea9524165fd08fbfcde58c11d8d12168f7a9ad0c46c621a045b925074f732996a625b82a96e70ae7ab9effa29212644ee7d6d51f7b1a7bee2a832524306e63aee09a5f21d5c5570811bfb386f4722108dcafc4df345e4221590835cab594097c8e6b971901a95542b848d9e5411130b99f301b36451892942e732c6d258e2589a811be0670b6ba8446cce430ebc73f98e68991a5ae93b3664aba84446b32bad26c7f4b37630e3527b69488edf6684d8c6f83021f0b86c89c6ec299d585bc5fbe8d804f0652be0d5ab623f84e55e26b9b4f839ba7a002fbb7c0c7b5fafa4210f7149e4110fe206df8aca2e5ad5516f90110f842a08dc286880de0cc507d96583b7c4c2b2b25239e58f8e67509b32edb5bbf293c00a00000000000000000000000000000000000000000000000000000000000000000f842a08dc286880de0cc507d96583b7c4c2b2b25239e58f8e67509b32edb5bbf293c01a000009803a9c7ea6a200004000000000000000000000000000000000000000000f842a08dc286880de0cc507d96583b7c4c2b2b25239e58f8e67509b32edb5bbf293c02a00000000000000000000000000000000000000000000000000000000000000000f842a08dc286880de0cc507d96583b7c4c2b2b25239e58f8e67509b32edb5bbf293c03a0c5d2460186f7233c927e7db2dcc703c0e500b653ca82273b7bfad8045d85a470c0c0";
        // let block_raw = hex::decode(block_string).expect("block string decoding");

        let block = decode_block(block_string.into()).expect("block decoding");
    }

    #[test]
    fn test_verification() {
        // let block_string = "0xf915aaf90c6da0904e3f9205902a780563d861aaa9cd1d635597ad1893a92d7f83dc5fb51b6eb4a01dcc4de8dec75d7aab85b567b6ccd41ad312451b948a7413f0a142fd40d49347940000000000000000000000000000000000000000a0350f40f771a73cd6bda4c37283b88c771179469b07633568b6047cf649b8c7d1a05f25ec3493913aef80e3d1d99e653321be3db3b16c3c83b82e6081cdce66a55ca08d7a148023d3a4612e85b2f142dcec65c358ab7fbd3aebdfef6868c018d44e3cb901000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000083020000028347e7c48305802b1480a00000000000000000000000000000000000000000000000000000000000000000880000000000000000842de81128b902d20000000006000000080a0a0808080400000055fc28df8d54aef6ffb43695ef3fed55993ae008dc033c36bb48d3efa131e7d85c3e125f152e0b296a8fc6a7506c52ea7cf364a85d2f38425c495845b8e5fcaa621c65eed175759bed2c1293ba153fcbcc03a77ec5b8be6a70525284fadfe5ce2c40ed9e5cc59ec79c1f64558e7712705119f5b6adba6abe67b0a85eea6a57a372012d816767142d06b30a45da766d3453d5eea9f411e6ee25ab672da1580d09606162ddc947c873020f4e251c671b680146bffe0ce1a26d6a426f00a3206a7a59be7ae8bf499671db45344db4de2d610c2e7788dadf3c3798ffdaba155028566f1499f496b9fd0507954835c425270f3b08c68ab5475b7fea7dcb4de1fabb9b2c7ef6dcb3858f66ef02925afb5fddc6be130d470a7613058d3bf176cb5fa53d1b778f4f1f4b3b176af9fd234be9844f065a650a3facc3f18550948ae0eb8cb92a48a0a0413bd051054e27e359834a584c63da39cc094deac2c2290f60c6066b400efec9f3cbb4428d5972c2829a1dbaab6363507e1b4391210e26d3a03ada6e3384cf4db8a53f35bc30fbb1015104658e94160f29ac7becd4e75bfa8e46864216069023ab2a2c12a2be1c8ceed81c05645b3c6f9023339af49e62626e5b890c710c6d4c15c0b0fc72bcf3e609c3687b85ebde41101b1a7487d64a0edaf56bb9125ab05310b17001004453ad8e7ec56a804a97f9f39218488fbdcb6d90a7a95f1a965420c7cda79b8436cac411c00996466b19cb4591f17bba2014d0d7de150e6042fec3a7a55f4cc4114b8b85322d6694abbedd1c2acad8a2a41908f60d9ee30ed851b6a47e5ad2fe9aa22d5c26e11c2e4e04a76f8ebb6d8a7b7e265e9a032463508bbf4f7304b5d353b65bb49b22abbc3316675c793171e3d5234a5f71526d32e70de0056d4c9225dc65c35adbc36c2eb184db805662cb10e9ecb34553b909b4ac2e49eb5558dcf2d6fe86a0bff9d7d21f57c78bd2ed82e125e78f2c2ea91bf9079ae2a00785762a8d643f3892d163f783fe1d37e4e5cf63d2b08dff0dead8cdf0b7ce0080e2a00785762a8d643f3892d163f783fe1d37e4e5cf63d2b08dff0dead8cdf0b7ce0180e2a00785762a8d643f3892d163f783fe1d37e4e5cf63d2b08dff0dead8cdf0b7ce0280e2a00785762a8d643f3892d163f783fe1d37e4e5cf63d2b08dff0dead8cdf0b7ce0380e2a00785762a8d643f3892d163f783fe1d37e4e5cf63d2b08dff0dead8cdf0b7ce0480e2a00785762a8d643f3892d163f783fe1d37e4e5cf63d2b08dff0dead8cdf0b7ce8080e2a00785762a8d643f3892d163f783fe1d37e4e5cf63d2b08dff0dead8cdf0b7ce8180e2a00785762a8d643f3892d163f783fe1d37e4e5cf63d2b08dff0dead8cdf0b7ce8280e2a00785762a8d643f3892d163f783fe1d37e4e5cf63d2b08dff0dead8cdf0b7ce8380e2a00785762a8d643f3892d163f783fe1d37e4e5cf63d2b08dff0dead8cdf0b7ce8480e2a00785762a8d643f3892d163f783fe1d37e4e5cf63d2b08dff0dead8cdf0b7ce8580e2a00785762a8d643f3892d163f783fe1d37e4e5cf63d2b08dff0dead8cdf0b7ce8680f842a0274cde18dd9dbb04caf16ad5ee969c19fe6ca764d5688b5e1d419f4ac6cd1600a00000000000000000000000000000000000000000000000000000000000000000f842a0274cde18dd9dbb04caf16ad5ee969c19fe6ca764d5688b5e1d419f4ac6cd1601a032c649ae8d68e00d000000000000000000000000000000000000000000000000f842a0274cde18dd9dbb04caf16ad5ee969c19fe6ca764d5688b5e1d419f4ac6cd1602a00300000000000000000000000000000000000000000000000000000000000000f842a0bf101a6e1c8e83c11bd203a582c7981b91097ec55cbd344ce09005c1f26d1900a00000000000000000000000000000000000000000000000000000000000000000f842a0bf101a6e1c8e83c11bd203a582c7981b91097ec55cbd344ce09005c1f26d1901a0e703c84e676dc11b000000000000000000000000000000000000000000000000f842a0bf101a6e1c8e83c11bd203a582c7981b91097ec55cbd344ce09005c1f26d1902a00000000000000000000000000000000000000000000000000000000000000000f842a0bf101a6e1c8e83c11bd203a582c7981b91097ec55cbd344ce09005c1f26d1903a0c5d2460186f7233c927e7db2dcc703c0e500b653ca82273b7bfad8045d85a470e2a0bf101a6e1c8e83c11bd203a582c7981b91097ec55cbd344ce09005c1f26d190480e2a0cac9a3e8dd152c9b5f8abcd254f1abe57d4acde35cfe0f919b43e6f09307140080e2a0cac9a3e8dd152c9b5f8abcd254f1abe57d4acde35cfe0f919b43e6f09307140180e2a0cac9a3e8dd152c9b5f8abcd254f1abe57d4acde35cfe0f919b43e6f09307140280e2a0cac9a3e8dd152c9b5f8abcd254f1abe57d4acde35cfe0f919b43e6f09307140380e2a0cac9a3e8dd152c9b5f8abcd254f1abe57d4acde35cfe0f919b43e6f09307140480e2a0d141d84155cf135593f0ac888e4af96c360bbc4d82dd9164311b3932ab9b9b0080e2a0d141d84155cf135593f0ac888e4af96c360bbc4d82dd9164311b3932ab9b9b0180e2a0d141d84155cf135593f0ac888e4af96c360bbc4d82dd9164311b3932ab9b9b0280e2a0d141d84155cf135593f0ac888e4af96c360bbc4d82dd9164311b3932ab9b9b0380e2a0d141d84155cf135593f0ac888e4af96c360bbc4d82dd9164311b3932ab9b9b0480e2a0d141d84155cf135593f0ac888e4af96c360bbc4d82dd9164311b3932ab9b9b8080e2a0ddb1869fe308ddab3660375687fd2a3f94434c961ed68fc8beb6fc8459771c0080e2a0ddb1869fe308ddab3660375687fd2a3f94434c961ed68fc8beb6fc8459771c0180e2a0ddb1869fe308ddab3660375687fd2a3f94434c961ed68fc8beb6fc8459771c0280e2a0ddb1869fe308ddab3660375687fd2a3f94434c961ed68fc8beb6fc8459771c0380e2a0ddb1869fe308ddab3660375687fd2a3f94434c961ed68fc8beb6fc8459771c4080e2a0ddb1869fe308ddab3660375687fd2a3f94434c961ed68fc8beb6fc8459771c8080e2a0ddb1869fe308ddab3660375687fd2a3f94434c961ed68fc8beb6fc8459771c8180e2a0ddb1869fe308ddab3660375687fd2a3f94434c961ed68fc8beb6fc8459771c8280e2a0ddb1869fe308ddab3660375687fd2a3f94434c961ed68fc8beb6fc8459771c8380e2a0ddb1869fe308ddab3660375687fd2a3f94434c961ed68fc8beb6fc8459771c8c80e2a0ddb1869fe308ddab3660375687fd2a3f94434c961ed68fc8beb6fc8459771ca480e2a0ddb1869fe308ddab3660375687fd2a3f94434c961ed68fc8beb6fc8459771ca580e2a0ddb1869fe308ddab3660375687fd2a3f94434c961ed68fc8beb6fc8459771ca680e2a0ddb1869fe308ddab3660375687fd2a3f94434c961ed68fc8beb6fc8459771ca780e2a0ddb1869fe308ddab3660375687fd2a3f94434c961ed68fc8beb6fc8459771ca880e2a0ddb1869fe308ddab3660375687fd2a3f94434c961ed68fc8beb6fc8459771ca980e2a0ddb1869fe308ddab3660375687fd2a3f94434c961ed68fc8beb6fc8459771caa80e2a0ddb1869fe308ddab3660375687fd2a3f94434c961ed68fc8beb6fc8459771cab80f90936f8650384342770c08285fc9401020300000000000000000000000000000000008203e78025a08c5ae2492597dde3dcf5aecb0cff6ba3860d57c540ac1bcca3b129a2562c9ea2a04b8277e7629b7f0c29b2f4f598f5947a1ac2051b032418f886bd88f67407f9d8f8650484342770c08285fc9400000000000000000000000000000000000000008203e78025a0dcdad59185394a03a5ab978320b09e0f2b5c5c0aeef5a5a15a0147816043830fa004fb20787e200354df3d6fa40615f6d43c851222c0a06fda8e36940f8086dab9f8630584342770c0827850940000000000000000000000000000000000000000808025a037d860df9bfdcdedc84ad76dc2281c330f925b02eeff90b63162067b33abae07a07d0f5b4341b449320c59529a44ac98582c7957611b723f1fc686b8a3801d88bff86a0684342770c0832dc6c080109a6060604052600a8060106000396000f360606040526008565b0026a0e909f28a02715713732d38899d8dfe97688ffa3dc7a96a5072b367bac35badcba061e24f56eab4f791158b16ca771b7914d85d401f549618329624be3d546adef9f907940784342770c0832dc6c08020b9074260806040526040516100109061017b565b604051809103906000f08015801561002c573d6000803e3d6000fd5b506000806101000a81548173ffffffffffffffffffffffffffffffffffffffff021916908373ffffffffffffffffffffffffffffffffffffffff16021790555034801561007857600080fd5b5060008067ffffffffffffffff8111156100955761009461024a565b5b6040519080825280601f01601f1916602001820160405280156100c75781602001600182028036833780820191505090505b50905060008060009054906101000a900473ffffffffffffffffffffffffffffffffffffffff1690506020600083833c81610101906101e3565b60405161010d90610187565b61011791906101a3565b604051809103906000f080158015610133573d6000803e3d6000fd5b50600160006101000a81548173ffffffffffffffffffffffffffffffffffffffff021916908373ffffffffffffffffffffffffffffffffffffffff160217905550505061029b565b60d58061046783390190565b6102068061053c83390190565b61019d816101d9565b82525050565b60006020820190506101b86000830184610194565b92915050565b6000819050602082019050919050565b600081519050919050565b6000819050919050565b60006101ee826101ce565b826101f8846101be565b905061020381610279565b925060208210156102435761023e7fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff8360200360080261028e565b831692505b5050919050565b7f4e487b7100000000000000000000000000000000000000000000000000000000600052604160045260246000fd5b600061028582516101d9565b80915050919050565b600082821b905092915050565b6101bd806102aa6000396000f3fe608060405234801561001057600080fd5b506004361061002b5760003560e01c8063f566852414610030575b600080fd5b61003861004e565b6040516100459190610146565b60405180910390f35b6000600160009054906101000a900473ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff166381ca91d36040518163ffffffff1660e01b815260040160206040518083038186803b1580156100b857600080fd5b505afa1580156100cc573d6000803e3d6000fd5b505050506040513d601f19601f820116820180604052508101906100f0919061010a565b905090565b60008151905061010481610170565b92915050565b6000602082840312156101205761011f61016b565b5b600061012e848285016100f5565b91505092915050565b61014081610161565b82525050565b600060208201905061015b6000830184610137565b92915050565b6000819050919050565b600080fd5b61017981610161565b811461018457600080fd5b5056fea2646970667358221220a6a0e11af79f176f9c421b7b12f441356b25f6489b83d38cc828a701720b41f164736f6c63430008070033608060405234801561001057600080fd5b5060b68061001f6000396000f3fe6080604052348015600f57600080fd5b506004361060285760003560e01c8063ab5ed15014602d575b600080fd5b60336047565b604051603e9190605d565b60405180910390f35b60006001905090565b6057816076565b82525050565b6000602082019050607060008301846050565b92915050565b600081905091905056fea26469706673582212203a14eb0d5cd07c277d3e24912f110ddda3e553245a99afc4eeefb2fbae5327aa64736f6c63430008070033608060405234801561001057600080fd5b5060405161020638038061020683398181016040528101906100329190610063565b60018160001c6100429190610090565b60008190555050610145565b60008151905061005d8161012e565b92915050565b60006020828403121561007957610078610129565b5b60006100878482850161004e565b91505092915050565b600061009b826100f0565b91506100a6836100f0565b9250827fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff038211156100db576100da6100fa565b5b828201905092915050565b6000819050919050565b6000819050919050565b7f4e487b7100000000000000000000000000000000000000000000000000000000600052601160045260246000fd5b600080fd5b610137816100e6565b811461014257600080fd5b50565b60b3806101536000396000f3fe6080604052348015600f57600080fd5b506004361060285760003560e01c806381ca91d314602d575b600080fd5b60336047565b604051603e9190605a565b60405180910390f35b60005481565b6054816073565b82525050565b6000602082019050606d6000830184604d565b92915050565b600081905091905056fea26469706673582212209bff7098a2f526de1ad499866f27d6d0d6f17b74a413036d6063ca6a0998ca4264736f6c6343000807003326a066241a78c508f5786ee7778e264c2d55cf64e4036e8f17917e6db89666b2eec6a07b8f093a07a7a93e174c7dccd2a0833b5e9f608ba17b1f0d5a3da2a4164a0132c0";
        // let block = decode_block(block_string.into()).expect("block decoding");
        //
        // let parent_root = "323ce96c53ff0abf906b68e544885ca9798d0e042b690b76eefb7e9d9866db68";
        // let parent_root = hex::decode(parent_root).expect("parent root decoding");

        let block_string = "0xf915aaf90c6da0904e3f9205902a780563d861aaa9cd1d635597ad1893a92d7f83dc5fb51b6eb4a01dcc4de8dec75d7aab85b567b6ccd41ad312451b948a7413f0a142fd40d49347940000000000000000000000000000000000000000a0350f40f771a73cd6bda4c37283b88c771179469b07633568b6047cf649b8c7d1a05f25ec3493913aef80e3d1d99e653321be3db3b16c3c83b82e6081cdce66a55ca08d7a148023d3a4612e85b2f142dcec65c358ab7fbd3aebdfef6868c018d44e3cb901000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000083020000028347e7c48305802b1480a00000000000000000000000000000000000000000000000000000000000000000880000000000000000842de81128b902d20000000006000000080a0a0808080400000055fc28df8d54aef6ffb43695ef3fed55993ae008dc033c36bb48d3efa131e7d85c3e125f152e0b296a8fc6a7506c52ea7cf364a85d2f38425c495845b8e5fcaa621c65eed175759bed2c1293ba153fcbcc03a77ec5b8be6a70525284fadfe5ce2c40ed9e5cc59ec79c1f64558e7712705119f5b6adba6abe67b0a85eea6a57a372012d816767142d06b30a45da766d3453d5eea9f411e6ee25ab672da1580d09606162ddc947c873020f4e251c671b680146bffe0ce1a26d6a426f00a3206a7a59be7ae8bf499671db45344db4de2d610c2e7788dadf3c3798ffdaba155028566f1499f496b9fd0507954835c425270f3b08c68ab5475b7fea7dcb4de1fabb9b2c7ef6dcb3858f66ef02925afb5fddc6be130d470a7613058d3bf176cb5fa53d1b778f4f1f4b3b176af9fd234be9844f065a650a3facc3f18550948ae0eb8cb92a48a0a0413bd051054e27e359834a584c63da39cc094deac2c2290f60c6066b400efec9f3cbb4428d5972c2829a1dbaab6363507e1b4391210e26d3a03ada6e3384cf4db8a53f35bc30fbb1015104658e94160f29ac7becd4e75bfa8e46864216069023ab2a2c12a2be1c8ceed81c05645b3c6f9023339af49e62626e5b890c710c6d4c15c0b0fc72bcf3e609c3687b85ebde41101b1a7487d64a0edaf56bb9125ab05310b17001004453ad8e7ec56a804a97f9f39218488fbdcb6d90a7a95f1a965420c7cda79b8436cac411c00996466b19cb4591f17bba2014d0d7de150e6042fec3a7a55f4cc4114b8b85322d6694abbedd1c2acad8a2a41908f60d9ee30ed851b6a47e5ad2fe9aa22d5c26e11c2e4e04a76f8ebb6d8a7b7e265e9a032463508bbf4f7304b5d353b65bb49b22abbc3316675c793171e3d5234a5f71526d32e70de0056d4c9225dc65c35adbc36c2eb184db805662cb10e9ecb34553b909b4ac2e49eb5558dcf2d6fe86a0bff9d7d21f57c78bd2ed82e125e78f2c2ea91bf9079ae2a00785762a8d643f3892d163f783fe1d37e4e5cf63d2b08dff0dead8cdf0b7ce0080e2a00785762a8d643f3892d163f783fe1d37e4e5cf63d2b08dff0dead8cdf0b7ce0180e2a00785762a8d643f3892d163f783fe1d37e4e5cf63d2b08dff0dead8cdf0b7ce0280e2a00785762a8d643f3892d163f783fe1d37e4e5cf63d2b08dff0dead8cdf0b7ce0380e2a00785762a8d643f3892d163f783fe1d37e4e5cf63d2b08dff0dead8cdf0b7ce0480e2a00785762a8d643f3892d163f783fe1d37e4e5cf63d2b08dff0dead8cdf0b7ce8080e2a00785762a8d643f3892d163f783fe1d37e4e5cf63d2b08dff0dead8cdf0b7ce8180e2a00785762a8d643f3892d163f783fe1d37e4e5cf63d2b08dff0dead8cdf0b7ce8280e2a00785762a8d643f3892d163f783fe1d37e4e5cf63d2b08dff0dead8cdf0b7ce8380e2a00785762a8d643f3892d163f783fe1d37e4e5cf63d2b08dff0dead8cdf0b7ce8480e2a00785762a8d643f3892d163f783fe1d37e4e5cf63d2b08dff0dead8cdf0b7ce8580e2a00785762a8d643f3892d163f783fe1d37e4e5cf63d2b08dff0dead8cdf0b7ce8680f842a0274cde18dd9dbb04caf16ad5ee969c19fe6ca764d5688b5e1d419f4ac6cd1600a00000000000000000000000000000000000000000000000000000000000000000f842a0274cde18dd9dbb04caf16ad5ee969c19fe6ca764d5688b5e1d419f4ac6cd1601a032c649ae8d68e00d000000000000000000000000000000000000000000000000f842a0274cde18dd9dbb04caf16ad5ee969c19fe6ca764d5688b5e1d419f4ac6cd1602a00300000000000000000000000000000000000000000000000000000000000000f842a0bf101a6e1c8e83c11bd203a582c7981b91097ec55cbd344ce09005c1f26d1900a00000000000000000000000000000000000000000000000000000000000000000f842a0bf101a6e1c8e83c11bd203a582c7981b91097ec55cbd344ce09005c1f26d1901a0e703c84e676dc11b000000000000000000000000000000000000000000000000f842a0bf101a6e1c8e83c11bd203a582c7981b91097ec55cbd344ce09005c1f26d1902a00000000000000000000000000000000000000000000000000000000000000000f842a0bf101a6e1c8e83c11bd203a582c7981b91097ec55cbd344ce09005c1f26d1903a0c5d2460186f7233c927e7db2dcc703c0e500b653ca82273b7bfad8045d85a470e2a0bf101a6e1c8e83c11bd203a582c7981b91097ec55cbd344ce09005c1f26d190480e2a0cac9a3e8dd152c9b5f8abcd254f1abe57d4acde35cfe0f919b43e6f09307140080e2a0cac9a3e8dd152c9b5f8abcd254f1abe57d4acde35cfe0f919b43e6f09307140180e2a0cac9a3e8dd152c9b5f8abcd254f1abe57d4acde35cfe0f919b43e6f09307140280e2a0cac9a3e8dd152c9b5f8abcd254f1abe57d4acde35cfe0f919b43e6f09307140380e2a0cac9a3e8dd152c9b5f8abcd254f1abe57d4acde35cfe0f919b43e6f09307140480e2a0d141d84155cf135593f0ac888e4af96c360bbc4d82dd9164311b3932ab9b9b0080e2a0d141d84155cf135593f0ac888e4af96c360bbc4d82dd9164311b3932ab9b9b0180e2a0d141d84155cf135593f0ac888e4af96c360bbc4d82dd9164311b3932ab9b9b0280e2a0d141d84155cf135593f0ac888e4af96c360bbc4d82dd9164311b3932ab9b9b0380e2a0d141d84155cf135593f0ac888e4af96c360bbc4d82dd9164311b3932ab9b9b0480e2a0d141d84155cf135593f0ac888e4af96c360bbc4d82dd9164311b3932ab9b9b8080e2a0ddb1869fe308ddab3660375687fd2a3f94434c961ed68fc8beb6fc8459771c0080e2a0ddb1869fe308ddab3660375687fd2a3f94434c961ed68fc8beb6fc8459771c0180e2a0ddb1869fe308ddab3660375687fd2a3f94434c961ed68fc8beb6fc8459771c0280e2a0ddb1869fe308ddab3660375687fd2a3f94434c961ed68fc8beb6fc8459771c0380e2a0ddb1869fe308ddab3660375687fd2a3f94434c961ed68fc8beb6fc8459771c4080e2a0ddb1869fe308ddab3660375687fd2a3f94434c961ed68fc8beb6fc8459771c8080e2a0ddb1869fe308ddab3660375687fd2a3f94434c961ed68fc8beb6fc8459771c8180e2a0ddb1869fe308ddab3660375687fd2a3f94434c961ed68fc8beb6fc8459771c8280e2a0ddb1869fe308ddab3660375687fd2a3f94434c961ed68fc8beb6fc8459771c8380e2a0ddb1869fe308ddab3660375687fd2a3f94434c961ed68fc8beb6fc8459771c8c80e2a0ddb1869fe308ddab3660375687fd2a3f94434c961ed68fc8beb6fc8459771ca480e2a0ddb1869fe308ddab3660375687fd2a3f94434c961ed68fc8beb6fc8459771ca580e2a0ddb1869fe308ddab3660375687fd2a3f94434c961ed68fc8beb6fc8459771ca680e2a0ddb1869fe308ddab3660375687fd2a3f94434c961ed68fc8beb6fc8459771ca780e2a0ddb1869fe308ddab3660375687fd2a3f94434c961ed68fc8beb6fc8459771ca880e2a0ddb1869fe308ddab3660375687fd2a3f94434c961ed68fc8beb6fc8459771ca980e2a0ddb1869fe308ddab3660375687fd2a3f94434c961ed68fc8beb6fc8459771caa80e2a0ddb1869fe308ddab3660375687fd2a3f94434c961ed68fc8beb6fc8459771cab80f90936f8650384342770c08285fc9401020300000000000000000000000000000000008203e78025a08c5ae2492597dde3dcf5aecb0cff6ba3860d57c540ac1bcca3b129a2562c9ea2a04b8277e7629b7f0c29b2f4f598f5947a1ac2051b032418f886bd88f67407f9d8f8650484342770c08285fc9400000000000000000000000000000000000000008203e78025a0dcdad59185394a03a5ab978320b09e0f2b5c5c0aeef5a5a15a0147816043830fa004fb20787e200354df3d6fa40615f6d43c851222c0a06fda8e36940f8086dab9f8630584342770c0827850940000000000000000000000000000000000000000808025a037d860df9bfdcdedc84ad76dc2281c330f925b02eeff90b63162067b33abae07a07d0f5b4341b449320c59529a44ac98582c7957611b723f1fc686b8a3801d88bff86a0684342770c0832dc6c080109a6060604052600a8060106000396000f360606040526008565b0026a0e909f28a02715713732d38899d8dfe97688ffa3dc7a96a5072b367bac35badcba061e24f56eab4f791158b16ca771b7914d85d401f549618329624be3d546adef9f907940784342770c0832dc6c08020b9074260806040526040516100109061017b565b604051809103906000f08015801561002c573d6000803e3d6000fd5b506000806101000a81548173ffffffffffffffffffffffffffffffffffffffff021916908373ffffffffffffffffffffffffffffffffffffffff16021790555034801561007857600080fd5b5060008067ffffffffffffffff8111156100955761009461024a565b5b6040519080825280601f01601f1916602001820160405280156100c75781602001600182028036833780820191505090505b50905060008060009054906101000a900473ffffffffffffffffffffffffffffffffffffffff1690506020600083833c81610101906101e3565b60405161010d90610187565b61011791906101a3565b604051809103906000f080158015610133573d6000803e3d6000fd5b50600160006101000a81548173ffffffffffffffffffffffffffffffffffffffff021916908373ffffffffffffffffffffffffffffffffffffffff160217905550505061029b565b60d58061046783390190565b6102068061053c83390190565b61019d816101d9565b82525050565b60006020820190506101b86000830184610194565b92915050565b6000819050602082019050919050565b600081519050919050565b6000819050919050565b60006101ee826101ce565b826101f8846101be565b905061020381610279565b925060208210156102435761023e7fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff8360200360080261028e565b831692505b5050919050565b7f4e487b7100000000000000000000000000000000000000000000000000000000600052604160045260246000fd5b600061028582516101d9565b80915050919050565b600082821b905092915050565b6101bd806102aa6000396000f3fe608060405234801561001057600080fd5b506004361061002b5760003560e01c8063f566852414610030575b600080fd5b61003861004e565b6040516100459190610146565b60405180910390f35b6000600160009054906101000a900473ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff166381ca91d36040518163ffffffff1660e01b815260040160206040518083038186803b1580156100b857600080fd5b505afa1580156100cc573d6000803e3d6000fd5b505050506040513d601f19601f820116820180604052508101906100f0919061010a565b905090565b60008151905061010481610170565b92915050565b6000602082840312156101205761011f61016b565b5b600061012e848285016100f5565b91505092915050565b61014081610161565b82525050565b600060208201905061015b6000830184610137565b92915050565b6000819050919050565b600080fd5b61017981610161565b811461018457600080fd5b5056fea2646970667358221220a6a0e11af79f176f9c421b7b12f441356b25f6489b83d38cc828a701720b41f164736f6c63430008070033608060405234801561001057600080fd5b5060b68061001f6000396000f3fe6080604052348015600f57600080fd5b506004361060285760003560e01c8063ab5ed15014602d575b600080fd5b60336047565b604051603e9190605d565b60405180910390f35b60006001905090565b6057816076565b82525050565b6000602082019050607060008301846050565b92915050565b600081905091905056fea26469706673582212203a14eb0d5cd07c277d3e24912f110ddda3e553245a99afc4eeefb2fbae5327aa64736f6c63430008070033608060405234801561001057600080fd5b5060405161020638038061020683398181016040528101906100329190610063565b60018160001c6100429190610090565b60008190555050610145565b60008151905061005d8161012e565b92915050565b60006020828403121561007957610078610129565b5b60006100878482850161004e565b91505092915050565b600061009b826100f0565b91506100a6836100f0565b9250827fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff038211156100db576100da6100fa565b5b828201905092915050565b6000819050919050565b6000819050919050565b7f4e487b7100000000000000000000000000000000000000000000000000000000600052601160045260246000fd5b600080fd5b610137816100e6565b811461014257600080fd5b50565b60b3806101536000396000f3fe6080604052348015600f57600080fd5b506004361060285760003560e01c806381ca91d314602d575b600080fd5b60336047565b604051603e9190605a565b60405180910390f35b60005481565b6054816073565b82525050565b6000602082019050606d6000830184604d565b92915050565b600081905091905056fea26469706673582212209bff7098a2f526de1ad499866f27d6d0d6f17b74a413036d6063ca6a0998ca4264736f6c6343000807003326a066241a78c508f5786ee7778e264c2d55cf64e4036e8f17917e6db89666b2eec6a07b8f093a07a7a93e174c7dccd2a0833b5e9f608ba17b1f0d5a3da2a4164a0132c0";
        let block = decode_block(block_string.into()).expect("block decoding");

        let parent_root = "323ce96c53ff0abf906b68e544885ca9798d0e042b690b76eefb7e9d9866db68";
        let parent_root = hex::decode(parent_root).expect("parent root decoding");

        verification(block, &parent_root).expect("Verification failed");
    }
}