create table
    todos (
        id serial primary key,
        title text not null,
        created_at timestamp default current_timestamp
    );