use crate::cache::extensions_base::{CacheExtDefault, DBCache};
use crate::utils::errors::ApiResult;
use rocket::response::content;
use serde::Serialize;

pub trait InfoProviderCache {
    fn db_id() -> usize {
        1
    }
}
pub trait GeneralCache {
    fn db_id() -> usize {
        0
    }
}

impl<T: DBCacheRequest + ?Sized> InfoProviderCache for T {}

impl<T: DBCacheRequest + ?Sized> GeneralCache for T {}

pub trait DBCacheRequest: CacheExtDefault + DBCache {
    fn cache_resp<R>(
        &self,
        key: &str,
        timeout: usize,
        resp: impl Fn() -> ApiResult<R>,
    ) -> ApiResult<content::Json<String>>
    where
        R: Serialize,
    {
        Self::select_database(self);
        Self::cache_resp_internal(self, key, timeout, resp)
    }

    fn request_cached(
        &self,
        client: &reqwest::blocking::Client,
        url: &str,
        timeout: usize,
        error_timeout: usize,
    ) -> ApiResult<String> {
        Self::select_database(self);
        Self::request_cached_internal(self, client, url, timeout, error_timeout)
    }

    fn invalidate_pattern(pattern: &str) {}

    fn invalidate_caches(pattern: &str) {}
}
