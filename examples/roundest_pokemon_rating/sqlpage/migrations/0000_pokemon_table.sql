CREATE TABLE IF NOT EXISTS pokemon (
    dex_id INTEGER PRIMARY KEY,
    name TEXT NOT NULL,
    up_votes INTEGER DEFAULT 0,
    down_votes INTEGER DEFAULT 0
);