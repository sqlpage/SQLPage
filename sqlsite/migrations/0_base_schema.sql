CREATE TABLE component(
    name TEXT PRIMARY KEY,
    description TEXT
);

CREATE TABLE parameter(
    name TEXT PRIMARY KEY,
    component TEXT REFERENCES component(name),
    description TEXT,
    type TEXT,
    optional BOOL DEFAULT FALSE
);

CREATE TABLE example_value(
    component TEXT REFERENCES component(name),
    parameter TEXT REFERENCES parameter(name),
    value TEXT
);