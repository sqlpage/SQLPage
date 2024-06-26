CREATE TABLE IF NOT EXISTS "currencies" (
    "id"     INTEGER PRIMARY KEY AUTOINCREMENT,
    "name"   TEXT COLLATE NOCASE NOT NULL UNIQUE,
    "to_rub" REAL NOT NULL
);

INSERT OR IGNORE INTO "currencies"("name", "to_rub") VALUES
    ('RUR', 1),
    ('USD', 90),
    ('CNY', 12.34);