table! {
    person (person_id) {
        person_id -> Int4,
        first_name -> Varchar,
        last_name -> Varchar,
        created_dt -> Timestamp,
        updated_dt -> Nullable<Timestamp>,
        deleted_dt -> Nullable<Timestamp>,
    }
}
