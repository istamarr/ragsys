use futures::TryStreamExt;
use ipfs_api::{IpfsApi, IpfsClient};
use std::io::{Write};
use std::{env, io};
use serde::Deserialize;
use std::path::Path;
use log::{info};
use warp::{Rejection, Reply};
use tokio::fs::File;
use anyhow::{Result};
use reqwest;
use reqwest::{multipart};
use serde_json::json;
use actix_web::http::StatusCode;
use actix_web::HttpResponse;
use tokio_util::codec::{BytesCodec, FramedRead};

#[derive(Clone, Debug, Deserialize)]
pub struct IpfsQuery {
    content: String,
}

pub fn dir_ipfs_upload_report(){
    info!("Upload report ");
}

pub async fn dir_ipfs_upload_image(name_file:String,flag:String){
    info!("Upload image");
    let path_dir = format!("/dir/img/{:?}",name_file).to_string();
    let client = IpfsClient::default();
    match client
        .get(&*path_dir)
        .map_ok(|chunk| chunk.to_vec())
        .try_concat()
        .await
    {
        Ok(res) => {
            let out = io::stdout();
            let mut out = out.lock();
            out.write_all(&res).unwrap();

            info!("Image upload success");
        }
        Err(e) => eprintln!("error getting file: {}", e)
    }
    //upload--->show & CID--->not publish without flag
    //flag:
    //1: publish peer
    //0: not publish peer
}

pub async fn ipfs_handler_upload_file(uid : String,
                                       body : RequestBody,
                                       info: IpfsQuery)
                                  -> Result<impl Reply, Rejection> {
     info!("Upload IPFS Per File");
     let result = some_async_call(info).await;
     match result {
         Ok(data) => {
             let body = warp::reply::json(&json!({ "message": "Success" }));
             Ok(warp::reply::with_status(body, StatusCode::OK))
         }
         Err(_) => {
             let body = json!({ "error": "Something went wrong" });
             Ok(warp::reply::with_status(warp::reply::json(&body), StatusCode::INTERNAL_SERVER_ERROR))
         }
     }

}

#[derive(Clone, Debug, Deserialize)]
pub struct RequestBody{
    pub model : String,
    pub prompt : String,
    pub options : String,
    pub keepAlive : String,
}


pub async fn some_async_call(info: IpfsQuery) -> Result<HttpResponse, reqwest::Error> {
    println!(" Starting IPFS upload...");

    let ipfs_url = env::var("IPFS_URL").unwrap_or_else(|_| "http://127.0.0.1:5001".to_string());
    let upload_url = format!("{}/api/v0/add?wrap-with-directory=false", ipfs_url);

    let file_path = Path::new("file_ipfs/upload.txt");
    let file = match File::open(&file_path).await {
        Ok(f) => f,
        Err(e) => {
            eprintln!(" Failed to open file: {}", e);
            return Ok(HttpResponse::InternalServerError()
                .body(format!("Failed to open file: {}", e)));
        }
    };
    info!("<UNK> Starting IPFS upload...");

    let stream = FramedRead::new(file, BytesCodec::new());
    // let stream = futures_util::stream::iter(stream);
    let req_stream = reqwest::Body::wrap_stream(stream);
    let file_part = multipart::Part::stream(req_stream)
        .file_name("file_ipfs/upload.txt")
        .mime_str("text/plain")
        .unwrap();
    let form = multipart::Form::new().part("file", file_part);


    let client = reqwest::Client::new();
    let response = client.post(&upload_url).multipart(form).send().await?;
    let body = response.text().await?;
    info!("Raw IPFS Response:\n{}", body);

    Ok(HttpResponse::Ok().body(format!("IPFS Upload Complete!\n Response:\n{}", body)))
}
