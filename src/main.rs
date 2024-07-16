use actix_files::NamedFile;
use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::env;
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

#[derive(Debug, Deserialize)]
struct RpcResponse {
    result: Option<String>,
    error: Option<serde_json::Value>,
    id: String,
}

async fn get_new_address(client: web::Data<Client>, rpc_url: web::Data<String>) -> impl Responder {
    println!("Accessed /new-address");
    let rpc_user = "marachain";
    let rpc_pass = "marachain";
    let rpc_request = serde_json::json!({
        "jsonrpc": "1.0",
        "id": "curltest",
        "method": "getnewaddress",
        "params": ["voucheraddr"]
    });

    let response = client
        .post(rpc_url.get_ref())
        .basic_auth(rpc_user, Some(rpc_pass))
        .header("Content-Type", "application/json")
        .json(&rpc_request)
        .send()
        .await;

    match response {
        Ok(resp) => {
            if resp.status().is_success() {
                match resp.json::<RpcResponse>().await {
                    Ok(rpc_response) => {
                        if let Some(error) = rpc_response.error {
                            println!("RPC error: {:?}", error);
                            HttpResponse::InternalServerError().json(error)
                        } else if let Some(result) = rpc_response.result {
                            let new_address = NewAddressResponse { address: result };
                            HttpResponse::Ok().json(new_address)
                        } else {
                            println!("Unexpected RPC response: {:?}", rpc_response);
                            HttpResponse::InternalServerError().body("Unexpected RPC response")
                        }
                    }
                    Err(e) => {
                        println!("Failed to parse RPC response: {}", e);
                        HttpResponse::InternalServerError().body("Failed to parse RPC response")
                    }
                }
            } else {
                let status = resp.status();
                let text = resp
                    .text()
                    .await
                    .unwrap_or_else(|_| "Unknown error".to_string());
                println!(
                    "RPC request failed with status: {} and body: {}",
                    status, text
                );
                HttpResponse::InternalServerError().body(format!(
                    "RPC request failed with status: {} and body: {}",
                    status, text
                ))
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

    // Get RPC URL from environment variable or use a default
    let rpc_url = env::var("RPC_URL").unwrap_or_else(|_| "http://marachain:18332/".to_string());

    println!("Starting server at http://0.0.0.0:8080");
    println!("Using RPC URL: {}", rpc_url);

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(client.clone()))
            .app_data(web::Data::new(rpc_url.clone()))
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
