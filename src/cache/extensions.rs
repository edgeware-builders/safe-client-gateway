use crate::cache::cache::{Cache, CACHE_REQS_PREFIX, CACHE_RESP_PREFIX, CacheExt};
use crate::cache::cache_with_code::CachedWithCode;
use crate::cache::logic::{cache_resp, request_cached};
use crate::utils::errors::{ApiError, ApiResult};
use rocket::response::content;
use serde::Serialize;

pub trait InfoCache: CacheExt {
    fn db_index(&self) -> usize { 2 }
}

impl<T: CacheExt + ?Sized> InfoCache for T {}
