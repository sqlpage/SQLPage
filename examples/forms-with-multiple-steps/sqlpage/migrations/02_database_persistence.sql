-- this table will store partially filled user forms
create table partially_filled_users (
    id integer primary key autoincrement,
    name text null, -- all fields are nullable, because the user may not have filled them yet
    email text null,
    age integer null check(age > 0)
);
