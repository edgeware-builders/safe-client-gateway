use crate::cache::cache_with_code::CachedWithCode;
use crate::cache::logic::{cache_resp, request_cached};
use crate::config::redis_scan_count;
use crate::utils::errors::{ApiError, ApiResult};
use mockall::automock;
use rocket::response::content;
use rocket_contrib::databases::redis::{
    self, pipe, Commands, FromRedisValue, Iter, PipelineCommands, ToRedisArgs,
};
use serde::ser::Serialize;
use serde_json;

pub const CACHE_RESP_PREFIX: &'static str = "c_resp";
pub const CACHE_REQS_PREFIX: &'static str = "c_reqs";

#[database("service_cache")]
pub struct ServiceCache(redis::Connection);

#[automock]
pub trait Cache {
    fn fetch(&self, id: &str) -> Option<String>;
    fn create(&self, id: &str, dest: &str, timeout: usize);
    fn insert_in_hash(&self, hash: &str, id: &str, dest: &str);
    fn get_from_hash(&self, hash: &str, id: &str) -> Option<String>;
    fn has_key(&self, id: &str) -> bool;
    fn expire_entity(&self, id: &str, timeout: usize);
    fn invalidate_pattern(&self, pattern: &str);
    fn invalidate(&self, id: &str);
    fn info(&self) -> Option<String>;
}

impl Cache for ServiceCache {
    fn fetch(&self, id: &str) -> Option<String> {
        match self.get(id) {
            Ok(value) => Some(value),
            _ => None,
        }
    }

    fn create(&self, id: &str, dest: &str, timeout: usize) {
        let _: () = self.set_ex(id, dest, timeout).unwrap();
    }

    fn insert_in_hash(&self, hash: &str, id: &str, dest: &str) {
        let _: () = self.hset(hash, id, dest).unwrap();
    }

    fn get_from_hash(&self, hash: &str, id: &str) -> Option<String> {
        self.hget(hash, id).ok()
    }

    fn has_key(&self, id: &str) -> bool {
        let result: Option<usize> = self.exists(id).ok();
        result.map(|it| it != 0).unwrap_or(false)
    }

    fn expire_entity(&self, id: &str, timeout: usize) {
        let _: () = self.expire(id, timeout).unwrap();
    }

    fn invalidate_pattern(&self, pattern: &str) {
        pipeline_delete(self, scan_match_count(self, pattern, redis_scan_count()));
    }

    fn invalidate(&self, id: &str) {
        let _: () = self.del(id).unwrap();
    }

    fn info(&self) -> Option<String> {
        info(self)
    }
}

pub trait CacheExt: Cache {
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

    fn request_cached(
        &self,
        client: &reqwest::blocking::Client,
        url: &str,
        timeout: usize,
        error_timeout: usize,
    ) -> ApiResult<String> {
        request_cached(self, client, url, timeout, error_timeout)
    }
}

impl<T: Cache + ?Sized> CacheExt for T {}

fn pipeline_delete(con: &redis::Connection, keys: Iter<String>) {
    let pipeline = &mut pipe();
    for key in keys {
        pipeline.del(key);
    }
    pipeline.execute(con);
}

fn scan_match_count<P: ToRedisArgs, C: ToRedisArgs, RV: FromRedisValue>(
    con: &redis::Connection,
    pattern: P,
    count: C,
) -> redis::Iter<RV> {
    redis::cmd("SCAN")
        .cursor_arg(0)
        .arg("MATCH")
        .arg(pattern)
        .arg("COUNT")
        .arg(count)
        .iter(con)
        .unwrap()
}

fn info(con: &redis::Connection) -> Option<String> {
    redis::cmd("INFO").query(con).ok()
}
