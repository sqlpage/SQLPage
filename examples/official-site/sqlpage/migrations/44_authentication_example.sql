create table users (
    username text primary key,
    password_hash text not null,
    role text not null
);

-- Create example users with trivial passwords for the website's demo
insert into users (username, password_hash, role)
values
  ('admin', '$argon2i$v=19$m=8,t=1,p=1$YWFhYWFhYWE$ROyXNhK0utkzTA', 'admin'), -- password: admin
  ('user', '$argon2i$v=19$m=8,t=1,p=1$YWFhYWFhYWE$qsrWdjgl96ooYw', 'user'); -- password: user
-- (the password hashes can be generated using the `sqlpage.hash_password` function)

create table user_sessions (
    session_token text primary key,
    username text not null references users(username),
    created_at timestamp not null default current_timestamp
);