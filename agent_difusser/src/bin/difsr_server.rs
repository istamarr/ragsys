use agent_difusser::srv::http_server;

#[tokio::main]
async fn main() {
    env_logger::init();
    
    let port: u16 = std::env::var("DIFSR_PORT")
        .unwrap_or_else(|_| "8081".to_string())
        .parse()
        .unwrap_or(8081);
    
    http_server::start_server(port).await;
}
