mod cli;
mod config;
mod types;
mod verkle;
mod dot;

pub use crate::config::Config;
pub use self::cli::Args;
use actix_web::{web::{self, Json}, App, HttpServer};
use types::{VerkleReq, VerkleResp};

async fn get_block_info(data: Json<VerkleReq>) -> Result<Json<VerkleResp>, actix_web::Error> {
    let block_number = data.into_inner().block_number;
    let block_rlp = verkle::get_rlp(block_number).await.unwrap();
    let block = verkle::decode_block(block_rlp.clone()).unwrap();

    // verkle::save_rlp(block_rlp.clone(), block_number).await.unwrap();
    verkle::print_block_info(&block);

    if block_number < 2 {
        return Ok(Json(VerkleResp { block_rlp: "incorrect block number".to_owned() }));
    }

    let previous_block_rlp = verkle::get_rlp(block_number - 1).await.unwrap();
    let previous_block = verkle::decode_block(previous_block_rlp.clone()).unwrap();

    let parent_root = hex::encode(previous_block.header.storage_root);

    match verkle::verification(block, parent_root) {
        Ok(val) => {
            dot::to_dot(&val);

            Ok(Json(VerkleResp { block_rlp }))
        },
        Err(err) => {
            log::error!("Error : {}", err);
            Ok(Json(VerkleResp { block_rlp: "error with verification".to_owned() }))
        }
    }
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
