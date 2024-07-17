use actix_files::NamedFile;
use actix_web::error::ErrorBadRequest;
use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::env;
use std::path::PathBuf;

#[derive(Debug, Deserialize)]
struct Label {
    label: String,
}

async fn list_addresses_by_label(
    client: &Client,
    label: &str,
    rpc_url: &str,
    rpc_user: &str,
    rpc_pass: &str,
) -> Result<Vec<String>, reqwest::Error> {
    let rpc_request = serde_json::json!({
        "jsonrpc": "1.0",
        "id": "listaddressesbylabel",
        "method": "getaddressesbylabel",
        "params": [label]
    });

    let response = client
        .post(rpc_url)
        .basic_auth(rpc_user, Some(rpc_pass))
        .header("Content-Type", "application/json")
        .json(&rpc_request)
        .send()
        .await?;

    let response_json: serde_json::Value = response.json().await?;
    let addresses = response_json["result"]
        .as_object()
        .unwrap()
        .keys()
        .cloned()
        .collect::<Vec<_>>();
    Ok(addresses)
}

async fn list_unspent_outputs(
    client: &Client,
    addresses: Vec<String>,
    rpc_url: &str,
    rpc_user: &str,
    rpc_pass: &str,
) -> Result<Vec<serde_json::Value>, reqwest::Error> {
    let rpc_request = serde_json::json!({
        "jsonrpc": "1.0",
        "id": "listunspent",
        "method": "listunspent",
        "params": [0, 9999999, addresses]
    });

    let response = client
        .post(rpc_url)
        .basic_auth(rpc_user, Some(rpc_pass))
        .header("Content-Type", "application/json")
        .json(&rpc_request)
        .send()
        .await?;

    let response_json: serde_json::Value = response.json().await?;
    let utxos = response_json["result"].as_array().unwrap().clone();
    Ok(utxos)
}

async fn create_and_send_raw_transaction(
    client: &Client,
    utxos: Vec<serde_json::Value>,
    destination_address: &str,
    fee: f64,
    rpc_url: &str,
    rpc_user: &str,
    rpc_pass: &str,
) -> Result<String, actix_web::error::Error> {
    let mut inputs = Vec::new();
    let mut total_amount = 0.0;

    for utxo in &utxos {
        let txid = utxo["txid"]
            .as_str()
            .ok_or_else(|| ErrorBadRequest("Missing txid"))?;
        let vout = utxo["vout"]
            .as_u64()
            .ok_or_else(|| ErrorBadRequest("Missing vout"))?;
        let amount = utxo["amount"]
            .as_f64()
            .ok_or_else(|| ErrorBadRequest("Missing amount"))?;
        total_amount += amount;

        inputs.push(json!({ "txid": txid, "vout": vout }));
    }

    let send_amount = total_amount - fee;
    if send_amount <= 0.0 {
        return Err(ErrorBadRequest(
            "Insufficient funds to cover the transaction fee",
        ));
    }

    let outputs = json!({ destination_address: send_amount });

    let rpc_request = json!({
        "jsonrpc": "1.0",
        "id": "createrawtransaction",
        "method": "createrawtransaction",
        "params": [inputs, outputs]
    });

    let raw_tx_response = client
        .post(rpc_url)
        .basic_auth(rpc_user, Some(rpc_pass))
        .header("Content-Type", "application/json")
        .json(&rpc_request)
        .send()
        .await
        .map_err(|e| ErrorBadRequest(e.to_string()))?;

    let raw_tx_json: serde_json::Value = raw_tx_response
        .json()
        .await
        .map_err(|e| ErrorBadRequest(e.to_string()))?;
    let raw_tx = raw_tx_json["result"]
        .as_str()
        .ok_or_else(|| ErrorBadRequest("Failed to create raw transaction"))?;

    let sign_rpc_request = json!({
        "jsonrpc": "1.0",
        "id": "signrawtransactionwithwallet",
        "method": "signrawtransactionwithwallet",
        "params": [raw_tx]
    });

    let sign_response = client
        .post(rpc_url)
        .basic_auth(rpc_user, Some(rpc_pass))
        .header("Content-Type", "application/json")
        .json(&sign_rpc_request)
        .send()
        .await
        .map_err(|e| ErrorBadRequest(e.to_string()))?;

    let sign_json: serde_json::Value = sign_response
        .json()
        .await
        .map_err(|e| ErrorBadRequest(e.to_string()))?;
    let signed_tx = sign_json["result"]["hex"]
        .as_str()
        .ok_or_else(|| ErrorBadRequest("Failed to sign transaction"))?;

    let send_rpc_request = json!({
        "jsonrpc": "1.0",
        "id": "sendrawtransaction",
        "method": "sendrawtransaction",
        "params": [signed_tx]
    });

    let send_response = client
        .post(rpc_url)
        .basic_auth(rpc_user, Some(rpc_pass))
        .header("Content-Type", "application/json")
        .json(&send_rpc_request)
        .send()
        .await
        .map_err(|e| ErrorBadRequest(e.to_string()))?;

    let send_json: serde_json::Value = send_response
        .json()
        .await
        .map_err(|e| ErrorBadRequest(e.to_string()))?;
    let txid = send_json["result"]
        .as_str()
        .ok_or_else(|| ErrorBadRequest("Failed to send transaction"))?
        .to_string();

    Ok(txid)
}
#[derive(Debug, Deserialize)]
struct SendAllRequest {
    label: String,
    destination_address: String,
    fee: f64,
}

async fn send_all(
    client: web::Data<Client>,
    rpc_url: web::Data<String>,
    req: web::Json<SendAllRequest>,
) -> impl Responder {
    let rpc_user = "marachain";
    let rpc_pass = "marachain";

    // List addresses by label
    let addresses =
        match list_addresses_by_label(&client, &req.label, &rpc_url, rpc_user, rpc_pass).await {
            Ok(addresses) => addresses,
            Err(e) => {
                return HttpResponse::InternalServerError()
                    .body(format!("Failed to list addresses: {}", e))
            }
        };

    // List unspent outputs
    let utxos = match list_unspent_outputs(&client, addresses, &rpc_url, rpc_user, rpc_pass).await {
        Ok(utxos) => utxos,
        Err(e) => {
            return HttpResponse::InternalServerError()
                .body(format!("Failed to list unspent outputs: {}", e))
        }
    };

    // Create and send raw transaction
    let txid = match create_and_send_raw_transaction(
        &client,
        utxos,
        &req.destination_address,
        req.fee,
        &rpc_url,
        rpc_user,
        rpc_pass,
    )
    .await
    {
        Ok(txid) => txid,
        Err(e) => {
            return HttpResponse::InternalServerError()
                .body(format!("Failed to create and send transaction: {}", e))
        }
    };

    HttpResponse::Ok().body(txid)
}

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

#[derive(Deserialize)]
struct VoucherCode {
    code: String,
}

async fn get_new_address(
    client: web::Data<Client>,
    rpc_url: web::Data<String>,
    code: web::Json<VoucherCode>,
) -> impl Responder {
    println!("Accessed /new-address");
    let rpc_user = "marachain";
    let rpc_pass = "marachain";
    let label = &code.code;
    let rpc_request = serde_json::json!({
        "jsonrpc": "1.0",
        "id": "curltest",
        "method": "getnewaddress",
        "params": [label]
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
            .route("/new-address", web::post().to(get_new_address))
            .route("/new-code", web::get().to(get_new_code))
            .route("/accept", web::post().to(accept_address_and_code))
            .route("/send-all", web::post().to(send_all))
            .service(actix_files::Files::new("/static", "./static").show_files_listing())
    })
    .bind("0.0.0.0:8080")?
    .run()
    .await
}
