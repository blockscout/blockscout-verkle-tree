mod cli;
mod config;
mod types;

pub use crate::config::Config;

pub use self::cli::Args;

use actix_web::{web::{self, Json}, App, HttpServer};

use types::{VerkleReq, VerkleResp};

async fn verkle(data: Json<VerkleReq>) -> Result<Json<VerkleResp>, actix_web::Error>{
    Ok(Json(VerkleResp { block_number: data.into_inner().block_number }))
}

pub async fn run(config: Config) -> std::io::Result<()> {
    let socket_addr = config.server.addr;

    log::info!("Server is starting at {}", socket_addr);
    HttpServer::new(move || {
        App::new().service(web::resource("/").route(web::post().to(verkle)))
    })
    .bind(socket_addr)?
    .run()
    .await
}
