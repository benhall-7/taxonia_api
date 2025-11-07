use poem_openapi::{payload::PlainText, OpenApi};

pub struct HealthCheckApi;

#[OpenApi]
impl HealthCheckApi {
    /// Health check endpoint
    #[oai(path = "/health_check", method = "get", operation_id = "health_check")]
    async fn health_check(&self) -> PlainText<String> {
        PlainText("OK".to_string())
    }
}
