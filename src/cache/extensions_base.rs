use crate::cache::cache::{Cache, CACHE_REQS_PREFIX, CACHE_RESP_PREFIX};
use crate::cache::cache_with_code::CachedWithCode;
use crate::utils::errors::{ApiError, ApiResult};
use rocket::response::content;
use serde::Serialize;

pub(super) trait DBCache: Cache {
    fn db_id() -> usize;

    fn select_database(self) {
        log::error!("SELECTED DB is {}", Self::db_id());
    }
}

pub(super) trait CacheExtDefault: Cache {
    fn invalidate_caches_internal(&self, key: &str) {
        self.invalidate_pattern(&format!("c_re*{}*", &key));
    }

    fn cache_resp_internal<R>(
        &self,
        key: &str,
        timeout: usize,
        resp: impl Fn() -> ApiResult<R>,
    ) -> ApiResult<content::Json<String>>
    where
        R: Serialize,
    {
        let cache_key = format!("{}_{}", CACHE_RESP_PREFIX, &key);
        let cached = self.fetch(&cache_key);
        match cached {
            Some(value) => Ok(content::Json(value)),
            None => {
                let resp = resp()?;
                let resp_string = serde_json::to_string(&resp)?;
                self.create(&cache_key, &resp_string, timeout);
                Ok(content::Json(resp_string))
            }
        }
    }

    fn request_cached_internal(
        &self,
        client: &reqwest::blocking::Client,
        url: &str,
        timeout: usize,
        error_timeout: usize,
    ) -> ApiResult<String> {
        let cache_key = format!("{}_{}", CACHE_REQS_PREFIX, &url);
        match self.fetch(&cache_key) {
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
                    self.create(
                        &cache_key,
                        CachedWithCode::join(status_code, &raw_data).as_str(),
                        error_timeout,
                    );
                    Err(ApiError::from_backend_error(status_code, &raw_data))
                } else {
                    self.create(
                        &cache_key,
                        CachedWithCode::join(status_code, &raw_data).as_str(),
                        timeout,
                    );
                    Ok(raw_data.to_string())
                }
            }
        }
    }
}
