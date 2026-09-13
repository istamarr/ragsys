use serde_json::Value;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = reqwest::Client::builder()
        .build()?;

    let data = "";

    let request = client.request(reqwest::Method::GET, "http://localhost/api/datatransaction/index.json")
        .body(data);

    let response = request.send().await?;
    let body = response.text().await?;

    println!("{}", body);

    let str_body = format!(r#"BODY{}"#,body);
    let check_final_response = serde_json::to_string(&*str_body).unwrap();
    println!("{}", check_final_response);

    Ok(())
}