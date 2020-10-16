extern crate actix_web;
extern crate postgres;

#[macro_use]
extern crate diesel;
extern crate r2d2;

#[macro_use]
extern crate actix_derive;

#[macro_use]
extern crate serde_derive;
extern crate serde_json;

mod data;
mod models;
mod schema;
mod web;

use crate::web::api::{
    create_person, delete_person, patch_person, query_person, subscribe_person, PersonSubscription,
};
use actix::Addr;
use actix_web::{App, HttpServer};
use diesel::r2d2::ConnectionManager;
use diesel::PgConnection;
use r2d2::Pool;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

struct AppState {
    subscriptions: RwLock<HashMap<i32, RwLock<Vec<Addr<PersonSubscription>>>>>,
}

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    dotenv::dotenv().ok();
    std::env::set_var("RUST_LOG", "actix_web=debug");
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    let manager = ConnectionManager::<PgConnection>::new(database_url);
    let pool: Pool<ConnectionManager<PgConnection>> = r2d2::Pool::builder()
        .build(manager)
        .expect("Failed to create pool.");

    let map: Arc<AppState> = Arc::new(AppState {
        subscriptions: RwLock::new(HashMap::new()),
    });

    HttpServer::new(move || {
        App::new()
            .data(pool.clone())
            .data(map.clone())
            .service(create_person)
            .service(query_person)
            .service(patch_person)
            .service(delete_person)
            .service(subscribe_person)
    })
    .bind("127.0.0.1:8081")?
    .run()
    .await
}
