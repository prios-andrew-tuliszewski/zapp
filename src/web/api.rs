use actix_web::{delete, get, post, web, Error, HttpResponse, Responder};
use diesel::r2d2::ConnectionManager;
use diesel::PgConnection;
use r2d2::Pool;

use crate::data::person::PersonRepo;

#[get("/")]
pub async fn hello() -> impl Responder {
    HttpResponse::Ok().body("Hello world!")
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreatePersonRequest {
    pub first_name: String,
    pub last_name: String,
    pub title: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct QueryPersonRequest {
    pub person_id: Option<i32>,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
}

#[post("/person")]
pub async fn create_person(
    db: web::Data<Pool<ConnectionManager<PgConnection>>>,
    item: web::Json<CreatePersonRequest>,
) -> Result<HttpResponse, Error> {
    Ok(web::block(move || PersonRepo::create_person(db, item.0))
        .await
        .map(|user| HttpResponse::Created().json(user))
        .map_err(|_| HttpResponse::InternalServerError())?)
}

#[get("/person")]
pub async fn query_person(
    db: web::Data<Pool<ConnectionManager<PgConnection>>>,
    item: web::Query<QueryPersonRequest>,
) -> Result<HttpResponse, Error> {
    Ok(web::block(move || PersonRepo::query_person(db, item.0))
        .await
        .map(|user| HttpResponse::Ok().json(user))
        .map_err(|_| HttpResponse::InternalServerError())?)
}

#[delete("/person/{id}")]
pub async fn delete_person(
    db: web::Data<Pool<ConnectionManager<PgConnection>>>,
    item: web::Path<i32>,
) -> Result<HttpResponse, Error> {
    Ok(web::block(move || PersonRepo::delete_person(db, item.0))
        .await
        .map(|user| HttpResponse::Ok().json(user))
        .map_err(|_| HttpResponse::InternalServerError())?)
}
