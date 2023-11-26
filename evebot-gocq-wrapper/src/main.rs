mod command;
mod constant;
mod error;
mod metadata;
mod server;
mod utils;

use actix_web::{web, App, HttpServer};
use server::main_handler;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| App::new().route("/", web::post().to(main_handler)))
        .bind(("127.0.0.1", 8080))?
        .run()
        .await
}
