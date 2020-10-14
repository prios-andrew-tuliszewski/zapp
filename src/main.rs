extern crate actix_web;
extern crate postgres;

#[macro_use]
extern crate diesel;
extern crate r2d2;

#[macro_use]
extern crate serde_derive;
extern crate serde_json;

mod data;
mod models;
mod schema;
mod web;

use crate::web::api::{create_person, query_person};
use actix_web::{App, HttpServer};
use diesel::r2d2::ConnectionManager;
use diesel::PgConnection;
use r2d2::Pool;

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    dotenv::dotenv().ok();
    std::env::set_var("RUST_LOG", "actix_web=debug");
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    let manager = ConnectionManager::<PgConnection>::new(database_url);
    let pool: Pool<ConnectionManager<PgConnection>> = r2d2::Pool::builder()
        .build(manager)
        .expect("Failed to create pool.");

    HttpServer::new(move || {
        App::new()
            .data(pool.clone())
            .service(create_person)
            .service(query_person)
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
