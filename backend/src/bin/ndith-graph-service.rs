use music_streaming_blocklist_backend::{run_service, ServiceMode};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    run_service(ServiceMode::Graph).await
}
