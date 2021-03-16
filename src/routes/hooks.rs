use crate::cache::cache::CacheExt;
use crate::config::webhook_token;
use crate::models::backend::webhooks::Payload;
use crate::services::hooks::invalidate_caches;
use crate::utils::context::Context;
use crate::utils::errors::ApiResult;
use rocket_contrib::json::Json;

#[post("/v1/hook/update/<token>", format = "json", data = "<update>")]
pub fn update(context: Context, token: String, update: Json<Payload>) -> ApiResult<()> {
    if token != webhook_token() {
        bail!("Invalid token");
    }
    invalidate_caches(context.cache(), &update)
}

#[get("/v1/flush_all/<token>")]
pub fn flush_all(context: Context, token: String) -> ApiResult<()> {
    if token != webhook_token() {
        bail!("Invalid token");
    }
    context.cache().flush_cache();
    Ok(())
}
