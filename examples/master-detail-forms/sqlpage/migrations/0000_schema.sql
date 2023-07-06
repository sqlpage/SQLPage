CREATE TABLE user(
    id INTEGER PRIMARY KEY,
    first_name TEXT NOT NULL,
    last_name TEXT NOT NULL,
    email TEXT
);

CREATE TABLE address(
    id INTEGER PRIMARY KEY,
    user_id INTEGER NOT NULL REFERENCES user(id),
    street TEXT NOT NULL,
    city TEXT NOT NULL,
    country TEXT NOT NULL
);