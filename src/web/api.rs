use actix::{Actor, Addr, Handler, StreamHandler};
use actix_web::{delete, get, patch, post, web, Error, HttpRequest, HttpResponse, Responder};
use actix_web_actors::ws;
use diesel::r2d2::ConnectionManager;
use diesel::PgConnection;
use r2d2::Pool;

use crate::data::person::{PersonError, PersonRepo};
use crate::models::{to_ws_person, Person, WsPerson, WsPerson2};
use crate::schema::person;
use crate::AppState;
use actix::prelude::Request;
use actix_web::body::Body;
use chrono::naive;
use std::borrow::{Borrow, BorrowMut};
use std::cell::RefCell;
use std::collections::hash_map::RandomState;
use std::collections::HashMap;
use std::ops::Deref;
use std::rc::Rc;
use std::sync::{Arc, RwLock, RwLockWriteGuard};

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

// pub fn test_move() {
//     web::block(move || {
//         let p = Person {
//             id: 0,
//             first_name: "".to_string(),
//             last_name: "".to_string(),
//             created_dt: chrono::Local::now().naive_local(),
//             updated_dt: None,
//             deleted_dt: None,
//             title: None,
//         };
//
//         let opt = Some(1);
//         opt.iter().for_each(|_| {
//             let c = p.clone();
//             let w = WsPerson {
//                 id: 0,
//                 first_name: c.first_name.into_boxed_str(),
//             };
//             move_ws_person(w);
//         });
//
//         Ok(p)
//     });
//     ()
// }

pub fn move_ws_person(ws_person: WsPerson) {}

#[patch("/person/{id}")]
pub async fn patch_person(
    db: web::Data<Pool<ConnectionManager<PgConnection>>>,
    state: web::Data<Arc<AppState>>,
    item: web::Json<PatchPersonRequest>,
    id: web::Path<i32>,
) -> Result<HttpResponse, Error> {
    Ok(web::block(move || {
        PersonRepo::patch_person(db, id.0, item.0).map(|person| {
            println!("Patched id {}", id.0);
            let ws_person = to_ws_person(person.clone());
            println!("Converted");
            let map = state.subscriptions.read().unwrap();
            let values = map.get(&id.0);
            println!("Got map {}", values.is_some());

            values.iter().for_each(|actors_rwl| {
                println!("Have a list of subs");
                actors_rwl.read().unwrap().deref().iter().for_each(|actor| {
                    println!("Sending to actor");
                    let ws2 = ws_person.clone();
                    let s = Box::new(ws2.first_name.as_ref().clone());
                    actor.do_send(WsPerson2 {
                        id: ws2.id,
                        first_name: "",
                    });

                    // println!("{:?}", request.);
                });
                ()
            });
            println!("Returning person");
            person
        })
    })
    .await
    .map(|user| HttpResponse::Created().json(user))
    .map_err(|_| HttpResponse::InternalServerError())?)

    // let person: Arc<Result<Person, PersonError>> = Arc::new(result);
    // let person_copy = Arc::clone(&person);
    //
    // let mut map: RwLockWriteGuard<
    //     HashMap<i32, RwLock<Vec<Addr<PersonSubscription>>>, RandomState>,
    // > = state.subscriptions.write().unwrap();
    // map.get(&id.0).iter().for_each(|actors| {
    //     actors.read().unwrap().deref().iter().for_each(|actor| {
    //         let _ = person_copy.iter().for_each(|p| {
    //             let m1: Person = p.clone();
    //             let _ = actor.send(m1);
    //             ()
    //         });
    //
    //         ()
    //     })
    // });
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
        println!("handling message");

        match msg {
            Ok(ws::Message::Ping(msg)) => ctx.pong(&msg),
            Ok(ws::Message::Text(text)) => ctx.text(text),
            Ok(ws::Message::Binary(bin)) => ctx.binary(bin),
            _ => (),
        }
    }
}

impl<'a> Handler<WsPerson2<'a>> for PersonSubscription {
    type Result = ();

    fn handle(&mut self, msg: WsPerson2, ctx: &mut Self::Context) -> Self::Result {
        println!("received message");
        let p_json = serde_json::to_string::<WsPerson2>(&msg);

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
