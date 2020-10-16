use std::borrow::{Borrow, BorrowMut};
use std::cell::RefCell;
use std::collections::hash_map::RandomState;
use std::collections::HashMap;
use std::ops::Deref;
use std::rc::Rc;
use std::sync::{Arc, RwLock, RwLockWriteGuard};

use actix::prelude::Request;
use actix::{Actor, Addr, Handler, StreamHandler};
use actix_web::body::Body;
use actix_web::{delete, get, patch, post, web, Error, HttpRequest, HttpResponse, Responder};
use actix_web_actors::ws;
use chrono::naive;
use diesel::r2d2::ConnectionManager;
use diesel::PgConnection;
use r2d2::Pool;

use crate::data::person::{PersonError, PersonRepo};
use crate::models::{to_ws_person, Person, WsPerson};
use crate::schema::person;
use crate::AppState;

#[derive(Debug, Serialize, Deserialize)]
pub struct CreatePersonRequest {
    pub first_name: String,
    pub last_name: String,
    pub title: Option<String>,
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

/*
====================================================================================================
 */

#[derive(Debug, Serialize, Deserialize)]
pub struct PatchPersonRequest {
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub title: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, AsChangeset)]
#[table_name = "person"]
pub struct PatchPersonRecord {
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub title: Option<String>,
    pub updated_dt: naive::NaiveDateTime,
}

#[patch("/person/{id}")]
pub async fn patch_person(
    db: web::Data<Pool<ConnectionManager<PgConnection>>>,
    state: web::Data<Arc<AppState>>,
    item: web::Json<PatchPersonRequest>,
    id: web::Path<i32>,
) -> Result<HttpResponse, Error> {
    Ok(web::block(move || {
        PersonRepo::patch_person(db, id.0, item.0).map(|person| {
            let map = state.subscriptions.read().unwrap();
            let values = map.get(&id.0);

            values.iter().for_each(|actors_rwl| {
                actors_rwl.read().unwrap().deref().iter().for_each(|actor| {
                    actor.do_send(person.clone());
                });
            });
            person
        })
    })
    .await
    .map(|user| HttpResponse::Created().json(user))
    .map_err(|_| HttpResponse::InternalServerError())?)
}

/*
====================================================================================================
 */

#[derive(Debug, Serialize, Deserialize)]
pub struct QueryPersonRequest {
    pub person_id: Option<i32>,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
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

/*
====================================================================================================
 */

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

/*
====================================================================================================
 */

#[derive(Clone, Copy)]
pub struct PersonSubscription;

impl Actor for PersonSubscription {
    type Context = ws::WebsocketContext<Self>;
}

/// Handler for ws::Message message
impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for PersonSubscription {
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        match msg {
            Ok(ws::Message::Ping(msg)) => ctx.pong(&msg),
            Ok(ws::Message::Text(text)) => ctx.text(text),
            Ok(ws::Message::Binary(bin)) => ctx.binary(bin),
            _ => (),
        }
    }
}

impl<'a> Handler<Person> for PersonSubscription {
    type Result = ();

    fn handle(&mut self, msg: Person, ctx: &mut Self::Context) -> Self::Result {
        println!("received message");
        let p_json = serde_json::to_string::<Person>(&msg);

        match p_json {
            Ok(json) => {
                println!("Sending {}", json);
                ctx.text(json)
            }
            Err(e) => println!("Failed {}", e),
        }
    }
}

#[get("/person/{id}/subscribe")]
pub async fn subscribe_person(
    req: HttpRequest,
    stream: web::Payload,
    state: web::Data<Arc<AppState>>,
    item: web::Path<i32>,
) -> impl Responder {
    let resp = match ws::start_with_addr(PersonSubscription {}, &req, stream) {
        Ok((addr, resp)) => {
            let mut map = state.subscriptions.write().unwrap();

            println!("Subscribing to {}", item.0);
            map.entry(item.0)
                .or_insert(RwLock::new(Vec::new()))
                .write()
                .unwrap()
                .push(addr);

            resp
        }
        Err(e) => HttpResponse::from_error(e),
    };

    println!("{:?}", resp);
    resp
}
