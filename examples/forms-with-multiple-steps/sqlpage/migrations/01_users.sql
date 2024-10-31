-- Simple SQLite users table
create table users (
    id integer primary key autoincrement,
    name text not null,
    email text not null,
    age integer not null check(age > 0)
);
