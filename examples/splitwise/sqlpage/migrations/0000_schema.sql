CREATE TABLE expense_group(
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT
);
CREATE TABLE group_member(
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    group_id INTEGER REFERENCES expense_group(id),
    name TEXT
);
CREATE TABLE expense(
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    spent_by INTEGER REFERENCES group_member(id),
    date TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    name TEXT,
    amount DECIMAL
);