mod cli;
mod config;
mod types;
mod verkle;

pub use crate::config::Config;
pub use self::cli::Args;
use actix_web::{web::{self, Json}, App, HttpServer};
use types::{VerkleReq, VerkleResp};

use ark_serialize::{CanonicalSerialize};

async fn get_block_info(data: Json<VerkleReq>) -> Result<Json<VerkleResp>, actix_web::Error> {
    let block_number = data.into_inner().block_number;
    let block_rlp = verkle::get_rlp(block_number).await.unwrap();

    verkle::save_rlp(block_rlp.clone(), block_number).await.unwrap();

    let block = verkle::decode_block(block_rlp.clone()).unwrap();

    verkle::print_block_info(&block);

    // for 462198 (dec)
    let info = verkle::verification(block, "4ab9e47a3b8508c264b56bca01a82ff0fa01dddb8e97d012058c49e133850539".to_owned());

    for (path, comm) in info.unwrap().commitments_by_path {
        println!("\tnode");
        let mut v = Vec::new();
        comm.serialize(&mut v).unwrap();
        println!("comm :\t{:?}", hex::encode(v));
        print!("path :\t");
        for el in path {
            print!("{:#02x} ", el);
        }
        println!();
    }

    Ok(Json(VerkleResp { block_rlp }))
}

pub async fn run(config: Config) -> std::io::Result<()> {
    let socket_addr = config.server.addr;

    log::info!("Server is starting at {}", socket_addr);
    HttpServer::new(move || {
        App::new().service(web::resource("/info").route(web::post().to(get_block_info)))
    })
    .bind(socket_addr)?
    .run()
    .await
}
