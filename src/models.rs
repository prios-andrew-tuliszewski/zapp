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
