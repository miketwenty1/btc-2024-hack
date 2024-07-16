use actix_files::NamedFile;
use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

async fn index() -> impl Responder {
    println!("Accessed /");
    let path: PathBuf = "./static/index.html".parse().unwrap();
    NamedFile::open(path)
}

#[derive(Serialize)]
struct NewAddressResponse {
    address: String,
}

#[derive(Serialize)]
struct NewCodeResponse {
    code: String,
}

#[derive(Deserialize)]
struct AcceptData {
    address: String,
    code: String,
}

#[derive(Deserialize)]
struct RpcResponse {
    result: String,
    error: Option<String>,
    id: String,
}

async fn get_new_address(client: web::Data<Client>) -> impl Responder {
    println!("Accessed /new-address");
    let rpc_url = "http://127.0.0.1:18332/";
    let rpc_user = "marachain";
    let rpc_pass = "marachain";
    let rpc_request = serde_json::json!({
        "jsonrpc": "1.0",
        "id": "curltest",
        "method": "getnewaddress",
        "params": ["voucheraddr"]
    });

    let response = client
        .post(rpc_url)
        .basic_auth(rpc_user, Some(rpc_pass))
        .header("Content-Type", "text/plain")
        .json(&rpc_request)
        .send()
        .await;

    match response {
        Ok(resp) => {
            if resp.status().is_success() {
                match resp.json::<RpcResponse>().await {
                    Ok(rpc_response) => {
                        if let Some(error) = rpc_response.error {
                            println!("RPC error: {}", error);
                            HttpResponse::InternalServerError().body(error)
                        } else {
                            let new_address = NewAddressResponse {
                                address: rpc_response.result,
                            };
                            HttpResponse::Ok().json(new_address)
                        }
                    }
                    Err(e) => {
                        println!("Failed to parse RPC response: {}", e);
                        HttpResponse::InternalServerError().body("Failed to parse RPC response")
                    }
                }
            } else {
                println!("RPC request failed with status: {}", resp.status());
                HttpResponse::InternalServerError().body("RPC request failed")
            }
        }
        Err(e) => {
            println!("Failed to send RPC request: {}", e);
            HttpResponse::InternalServerError().body("Failed to send RPC request")
        }
    }
}

async fn get_new_code() -> impl Responder {
    println!("Accessed /new-code");
    let new_code = NewCodeResponse {
        code: "newcode1234".to_string(),
    };
    HttpResponse::Ok().json(new_code)
}

async fn accept_address_and_code(data: web::Json<AcceptData>) -> impl Responder {
    println!("Accessed /accept");
    println!("Received address: {}", data.address);
    println!("Received code: {}", data.code);

    HttpResponse::Ok().body("Address and code accepted")
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let client = Client::new();

    println!("Starting server at http://0.0.0.0:8080");

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(client.clone()))
            .route("/", web::get().to(index))
            .route("/new-address", web::get().to(get_new_address))
            .route("/new-code", web::get().to(get_new_code))
            .route("/accept", web::post().to(accept_address_and_code))
            .service(actix_files::Files::new("/static", "./static").show_files_listing())
    })
    .bind("0.0.0.0:8080")?
    .run()
    .await
}
