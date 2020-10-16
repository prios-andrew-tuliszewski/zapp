use actix::prelude::*;
use chrono::naive;

#[derive(Queryable, Serialize)]
pub struct Person {
    pub id: i32,
    pub first_name: String,
    pub last_name: String,
    pub created_dt: naive::NaiveDateTime,
    pub updated_dt: Option<naive::NaiveDateTime>,
    pub deleted_dt: Option<naive::NaiveDateTime>,
    pub title: Option<String>,
}

#[derive(Message, Serialize, Clone)]
#[rtype(result = "()")]
pub struct WsPerson {
    pub id: i32,
    pub first_name: Box<str>,
    // pub last_name: &'a str,
    // pub created_dt: naive::NaiveDateTime,
    // pub updated_dt: Option<naive::NaiveDateTime>,
    // pub deleted_dt: Option<naive::NaiveDateTime>,
    // pub title: Option<&'a str>,
}

#[derive(Message, Serialize, Copy, Clone)]
#[rtype(result = "()")]
pub struct WsPerson2<'a> {
    pub id: i32,
    pub first_name: &'a str,
    // pub last_name: &'a str,
    // pub created_dt: naive::NaiveDateTime,
    // pub updated_dt: Option<naive::NaiveDateTime>,
    // pub deleted_dt: Option<naive::NaiveDateTime>,
    // pub title: Option<&'a str>,
}

pub fn to_ws_person<'a>(person: Person) -> Box<WsPerson> {
    Box::new(WsPerson {
        id: person.id,
        first_name: person.first_name.into_boxed_str(),
        // last_name: person.last_name.as_str(),
        // created_dt: person.created_dt.clone(),
        // updated_dt: person.updated_dt.clone(),
        // deleted_dt: person.deleted_dt.clone(),
        // title: person.title.as_deref(),
    })
}

impl Clone for Person {
    fn clone(&self) -> Person {
        Person {
            id: self.id,
            first_name: self.first_name.clone(),
            last_name: self.last_name.clone(),
            created_dt: self.created_dt.clone(),
            updated_dt: self.updated_dt.clone(),
            deleted_dt: self.deleted_dt.clone(),
            title: self.title.clone(),
        }
    }
}
