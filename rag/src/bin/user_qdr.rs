#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = reqwest::Client::builder()
        .build()?;

    let mut headers = reqwest::header::HeaderMap::new();
    headers.insert("Content-Type", "application/json".parse()?);

    let data = "";

    let request = client.request(reqwest::Method::GET,
        "http://localhost:6333/collections/asist_collection_detail/points/b559ee3f-4a97-4618-be61-5b7f2f6b6995")
        .headers(headers)
        .body(data);

    let response = request.send().await?;
    let body = response.text().await?;

    println!("{}", body);

    Ok(())
}
