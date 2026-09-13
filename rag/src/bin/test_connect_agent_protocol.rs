use rag::srv::srv_client::secure::agent_protocol_login::get_token_agent_protocol;

#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();
    env_logger::init();
    get_token_agent_protocol().await.expect("TODO: panic message");
}