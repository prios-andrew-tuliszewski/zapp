use actix_web::web;
use diesel::prelude::*;
use diesel::r2d2::ConnectionManager;
use diesel::{
    debug_query, delete, insert_into, BoolExpressionMethods, PgArrayExpressionMethods, QueryDsl,
    TextExpressionMethods,
};
use diesel::{PgConnection, RunQueryDsl};
use r2d2::{Pool, PooledConnection};
use std::error::Error;

use crate::models::Person;
use crate::schema::person;
use crate::schema::person::*;
use crate::web::api::{CreatePersonRequest, QueryPersonRequest};
use diesel::pg::Pg;

#[derive(Debug)]
pub struct PersonError {}

pub struct PersonRepo;

#[derive(Insertable, Queryable, Debug)]
#[table_name = "person"]
pub struct CreatePerson<'a> {
    pub first_name: &'a str,
    pub last_name: &'a str,
    pub title: Option<&'a str>,
    pub created_dt: chrono::NaiveDateTime,
}

impl PersonRepo {
    pub fn create_person(
        db: web::Data<Pool<ConnectionManager<PgConnection>>>,
        i: CreatePersonRequest,
    ) -> Result<Person, PersonError> {
        let conn = db.get().unwrap();

        let new_record = CreatePerson {
            first_name: &i.first_name,
            last_name: &i.last_name,
            title: i.title.as_deref(),
            created_dt: chrono::Local::now().naive_local(),
        };

        let q = insert_into(person::table).values(&new_record);

        println!("{}", debug_query::<Pg, _>(&q));
        let qr = q.get_result(&conn);

        qr.map_err(|_| PersonError {})
    }

    pub fn query_person(
        db: web::Data<Pool<ConnectionManager<PgConnection>>>,
        r: QueryPersonRequest,
    ) -> Result<Vec<Person>, PersonError> {
        let conn: PooledConnection<ConnectionManager<PgConnection>> = db.get().unwrap();

        match r.person_id {
            Some(id) => PersonRepo::query_person_by_id(conn, id),
            None => PersonRepo::query_person_by_names(conn, r.first_name, r.last_name),
        }
    }

    pub fn delete_person(
        db: web::Data<Pool<ConnectionManager<PgConnection>>>,
        id: i32,
    ) -> Result<usize, PersonError> {
        let conn: PooledConnection<ConnectionManager<PgConnection>> = db.get().unwrap();

        let q = delete(person::table.find(id));
        println!("{}", debug_query::<Pg, _>(&q));

        q.execute(&conn).map_err(|_| PersonError {})
    }

    fn query_person_by_id(
        conn: PooledConnection<ConnectionManager<PgConnection>>,
        id: i32,
    ) -> Result<Vec<Person>, PersonError> {
        let q = person::table.filter(person::person_id.eq(id));
        println!("{}", debug_query::<Pg, _>(&q));

        q.load::<Person>(&conn).map_err(|_| PersonError {})
    }

    fn query_person_by_names(
        conn: PooledConnection<ConnectionManager<PgConnection>>,
        f_name: Option<String>,
        l_name: Option<String>,
    ) -> Result<Vec<Person>, PersonError> {
        let result = match (f_name, l_name) {
            (Some(f), Some(l)) => {
                let q = person::table
                    .filter(first_name.like(f))
                    .filter(last_name.like(l));

                println!("{}", debug_query::<Pg, _>(&q));

                q.load::<Person>(&conn)
            }
            (Some(f), None) => {
                let q = person::table.filter(first_name.like(f));

                println!("{}", debug_query::<Pg, _>(&q));

                q.load::<Person>(&conn)
            }
            (None, Some(l)) => {
                let q = person::table.filter(last_name.like(l));
                println!("{}", debug_query::<Pg, _>(&q));

                q.load::<Person>(&conn)
            }
            (None, None) => person::table.load::<Person>(&conn),
        };

        result.map_err(|_| PersonError {})
    }
}
