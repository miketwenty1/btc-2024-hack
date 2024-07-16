use actix_files::NamedFile;
use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use std::path::PathBuf;

async fn index() -> impl Responder {
    let path: PathBuf = "./static/index.html".parse().unwrap();
    NamedFile::open(path)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| App::new().route("/", web::get().to(index)))
        .bind("0.0.0.0:9999")?
        .run()
        .await
}
