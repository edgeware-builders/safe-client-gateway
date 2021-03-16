use crate::cache::cache::{Cache, CACHE_REQS_PREFIX, CACHE_RESP_PREFIX};
use crate::cache::cache_with_code::CachedWithCode;
use crate::utils::errors::{ApiError, ApiResult};
use rocket::response::content;
use serde::Serialize;

// trait CacheExt: Cache {
//
// }
//
// impl<T: Cache + ?Sized> CacheExt for T {}
