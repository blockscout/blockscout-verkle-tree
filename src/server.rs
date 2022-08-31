use crate::verkle::{get_rlp, decode_block, verification};
use crate::dot::to_dot;
use actix_web::{web::self, App, HttpServer, HttpResponse};
use actix_web::http::{StatusCode};
use crate::types::{VerkleReq};
use crate::Config;
use tokio::process::Command;
use tempfile::tempdir;

async fn get_block_info(info: web::Path<VerkleReq>) -> Result<HttpResponse, crate::error::Error> {
    let block_number = info.block_number;
    let block_rlp = get_rlp(block_number).await?;
    let block = decode_block(block_rlp)?;

    if block_number < 2 {
        return Ok(HttpResponse::build(StatusCode::BAD_REQUEST)
                .content_type("text/html")
                .body("Incorrect block_number"));
    }

    let previous_block_rlp = get_rlp(block_number - 1).await?;
    let previous_block = decode_block(previous_block_rlp)?;

    let parent_root = previous_block.header.storage_root;
    let keyvals = block_verkle_proof_extractor::keyvals::KeyVals {
                    keys: block.header.keyvals.keys.clone(),
                    values: block.header.keyvals.values.clone()
                };

    let dir = tempdir()?;
    let file_path = dir.path().join("tmp.dot");

    match verification(block, &parent_root) {
        // FIX: to_dot lead to block :(
        Ok(val) => match to_dot(&val, &keyvals, &file_path) {
                        Ok(()) => {
                            let image_path = dir.path().join("tmp.svg");
                            let mut child = Command::new("dot")
                                .arg("-Tsvg")
                                .arg(&file_path)
                                .arg("-o")
                                .arg(&image_path)
                                .spawn()
                                .expect("failed to spawn");
                    
                            // Await until the command completes
                            let _status = child.wait().await?;
                            
                            // sending an image
                            // web::block ?
                            let image_content =  web::Bytes::from(std::fs::read(&image_path)?);

                            // should we drop files?
                            dir.close()?;

                            Ok(HttpResponse::build(StatusCode::OK)
                                .content_type("image/svg+xml")
                                .body(image_content))
                        },
                        Err(err) => Err(err.into())
                    },
        Err(err) => {
            tracing::error!("Error : {}", err);
            
            Ok(HttpResponse::build(StatusCode::INTERNAL_SERVER_ERROR)
                .content_type("text/html")
                .body("Error while verification"))
        }
    }
}

pub async fn run_http(config: Config) -> std::io::Result<()> {
    let socket_addr = config.server.addr;

    tracing::info!("Server is starting at {}", socket_addr);
    HttpServer::new(move || {
        App::new().service(web::resource("/block/{block_number}").route(web::get().to(get_block_info)))
    })
    .bind(socket_addr)?
    .run()
    .await
}
