use crate::verkle::{get_rlp, decode_block, print_block_info, verification};
use crate::dot::to_dot;
use actix_web::{web::{self, Json}, App, HttpServer};
use crate::types::{VerkleReq, VerkleResp};
use crate::Config;

async fn get_block_info(data: Json<VerkleReq>) -> Result<Json<VerkleResp>, crate::error::Error> {
    let block_number = data.into_inner().block_number;
    let block_rlp = get_rlp(block_number).await?;
    let block = decode_block(block_rlp.clone())?;

    // save_rlp(block_rlp.clone(), block_number).await?;
    print_block_info(&block);

    if block_number < 2 {
        return Ok(Json(VerkleResp { block_rlp: "incorrect block number".to_owned() }));
    }

    let previous_block_rlp = get_rlp(block_number - 1).await?;
    let previous_block = decode_block(previous_block_rlp)?;

    let parent_root = hex::encode(previous_block.header.storage_root);

    match verification(block, parent_root) {
        Ok(val) => match to_dot(&val, "example.dot") {
                       Ok(()) => Ok(Json(VerkleResp { block_rlp })),
                       Err(err) => Err(err.into())
                   },
        Err(err) => {
            log::error!("Error : {}", err);
            Ok(Json(VerkleResp { block_rlp: "error with verification".to_owned() }))
        }
    }
}

pub async fn run_http(config: Config) -> std::io::Result<()> {
    let socket_addr = config.server.addr;

    log::info!("Server is starting at {}", socket_addr);
    HttpServer::new(move || {
        App::new().service(web::resource("/info").route(web::post().to(get_block_info)))
    })
    .bind(socket_addr)?
    .run()
    .await
}
