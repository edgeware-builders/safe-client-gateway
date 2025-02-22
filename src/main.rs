#![feature(proc_macro_hygiene, decl_macro, option_result_contains)]

extern crate log;

#[macro_use]
extern crate rocket;

#[macro_use]
extern crate rocket_contrib;

extern crate dotenv;

#[macro_use]
pub mod macros;

mod cache;
mod config;
mod models;
mod monitoring;
mod providers;
mod routes;
mod services;
mod utils;

#[cfg(test)]
mod json;

use crate::routes::error_catchers;
use cache::redis::ServiceCache;
use dotenv::dotenv;
use routes::active_routes;
use std::time::Duration;
use utils::cors::CORS;

fn main() {
    dotenv().ok();
    env_logger::init();

    let client = reqwest::blocking::Client::builder()
        .connect_timeout(Duration::from_millis(
            config::internal_client_connect_timeout(),
        ))
        .build()
        .unwrap();

    rocket::ignite()
        .mount("/", active_routes())
        .manage(client)
        .attach(monitoring::performance::PerformanceMonitor())
        .attach(CORS())
        .attach(ServiceCache::fairing())
        .register(error_catchers())
        .launch();
}
