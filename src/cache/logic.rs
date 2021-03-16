use crate::cache::cache::{Cache, CacheExt, CACHE_REQS_PREFIX, CACHE_RESP_PREFIX};
use crate::cache::cache_with_code::CachedWithCode;
use crate::utils::errors::{ApiError, ApiResult};
use rocket::response::content;
use serde::Serialize;

pub(super) fn invalidate_caches(cache: &impl Cache, key: &str) {
    cache.invalidate_pattern(&format!("c_re*{}*", &key));
}

pub(super) fn cache_resp<C, R>(
    cache: &C,
    key: &str,
    timeout: usize,
    resp: impl Fn() -> ApiResult<R>,
) -> ApiResult<content::Json<String>>
where
    R: Serialize,
    C: Cache + ?Sized,
{
    let cache_key = format!("{}_{}", CACHE_RESP_PREFIX, &key);
    let cached = cache.fetch(&cache_key);
    match cached {
        Some(value) => Ok(content::Json(value)),
        None => {
            let resp = resp()?;
            let resp_string = serde_json::to_string(&resp)?;
            cache.create(&cache_key, &resp_string, timeout);
            Ok(content::Json(resp_string))
        }
    }
}

pub(super) fn request_cached<C>(
    cache: &C,
    client: &reqwest::blocking::Client,
    url: &str,
    timeout: usize,
    error_timeout: usize,
) -> ApiResult<String>
where
    C: Cache + ?Sized,
{
    let cache_key = format!("{}_{}", CACHE_REQS_PREFIX, &url);
    match cache.fetch(&cache_key) {
        Some(cached) => CachedWithCode::split(&cached).to_result(),
        None => {
            let response = client.get(url).send()?;
            let status_code = response.status().as_u16();

            // Early return and no caching if the error is a 500 or greater
            if response.status().is_server_error() {
                return Err(ApiError::from_backend_error(
                    42,
                    format!("Got server error for {}", response.text()?).as_str(),
                ));
            }

            let is_client_error = response.status().is_client_error();
            let raw_data = response.text()?;

            if is_client_error {
                cache.create(
                    &cache_key,
                    CachedWithCode::join(status_code, &raw_data).as_str(),
                    error_timeout,
                );
                Err(ApiError::from_backend_error(status_code, &raw_data))
            } else {
                cache.create(
                    &cache_key,
                    CachedWithCode::join(status_code, &raw_data).as_str(),
                    timeout,
                );
                Ok(raw_data.to_string())
            }
        }
    }
}
