use worker::*;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
struct HealthResponse {
    status: String,
    service: String,
    region: Option<String>,
}

#[event(fetch)]
pub async fn main(req: Request, env: Env, _ctx: Context) -> Result<Response> {
    console_error_panic_hook::set_once();

    Router::new()
        .get("/health", |_, ctx| async move {
            let region = ctx.env.var("CF_REGION").ok().map(|v| v.to_string());
            
            Response::from_json(&HealthResponse {
                status: "healthy".to_string(),
                service: "{{ service.name }}".to_string(),
                region,
            })
        })
        .run(req, env)
        .await
}