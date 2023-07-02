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
    -- identifiant du membre qui a fait la dépense
    date TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    -- date et heure de la dépense
    name TEXT,
    -- intitulé
    amount DECIMAL -- montant en euros
);