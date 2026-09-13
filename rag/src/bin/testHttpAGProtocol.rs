// #[tokio::main]
// async fn ag_pt_master_cmd() -> Result<(), Box<dyn std::error::Error>> {
//     let client = reqwest::Client::builder()
//         .build()?;
//
//     let mut headers = reqwest::header::HeaderMap::new();
//     headers.insert("Content-Type", "application/json".parse()?);
//     let data = r#"{
//     "title": "String",
//     "tags": [
//         "value1",
//         "value2"
//         ],
//         "data": "String",
//         "source": "String",
//         "date": "String"
//     }"#;
//
//     let json: serde_json::Value = serde_json::from_str(&data)?;
//
//     let request = client.request(reqwest::Method::POST, "http://localhost:9191/resource/master/flow/web")
//         .headers(headers)
//         .json(&json);
//
//     let response = request.send().await?;
//     let body = response.text().await?;
//
//     println!("{}", body);
//
//     Ok(())
// }

fn main() {}