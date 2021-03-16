use crate::cache::cache::{Cache, CACHE_REQS_PREFIX, CACHE_RESP_PREFIX};
use crate::cache::cache_with_code::CachedWithCode;
use crate::cache::logic::{cache_resp, request_cached};
use crate::utils::errors::{ApiError, ApiResult};
use rocket::response::content;
use serde::Serialize;

pub(super) trait CacheExt: Cache {
    //TODO
    fn invalidate_caches(&self, key: &str) {
        self.invalidate_pattern(&format!("c_re*{}*", &key));
    }

    fn cache_resp<R>(
        &self,
        key: &str,
        timeout: usize,
        resp: impl Fn() -> ApiResult<R>,
    ) -> ApiResult<content::Json<String>>
    where
        R: Serialize,
    {
        log::error!("Selecting database");
        self.select_db(10);
        cache_resp(self, key, timeout, resp)
    }

    fn request_cached(
        &self,
        client: &reqwest::blocking::Client,
        url: &str,
        timeout: usize,
        error_timeout: usize,
    ) -> ApiResult<String> {
        self.select_db(10);
        request_cached(self, client, url, timeout, error_timeout)
    }
}

impl<T: Cache + ?Sized> CacheExt for T {}
