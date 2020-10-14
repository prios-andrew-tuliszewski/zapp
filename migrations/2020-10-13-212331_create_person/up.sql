-- Your SQL goes here

CREATE TABLE person (
  person_id SERIAL PRIMARY KEY,
  first_name VARCHAR NOT NULL,
  last_name VARCHAR NOT NULL,
  created_dt TIMESTAMP NOT NULL DEFAULT current_timestamp,
  updated_dt TIMESTAMP,
  deleted_dt TIMESTAMP
)