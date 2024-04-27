create table todos(
    id integer primary key,
    title text not null,
    created_at timestamp default current_timestamp
);