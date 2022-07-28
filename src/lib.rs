mod cli;
mod config;
mod types;
mod verkle;

pub use crate::config::Config;
pub use self::cli::Args;
use actix_web::{web::{self, Json}, App, HttpServer};
use types::{VerkleReq, VerkleResp};

async fn get_block_info(data: Json<VerkleReq>) -> Result<Json<VerkleResp>, actix_web::Error> {
    let block_number = data.into_inner().block_number;
    let block_rlp = verkle::get_rlp(block_number).await.unwrap();

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
