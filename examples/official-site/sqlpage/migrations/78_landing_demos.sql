-- Sample data for the landing page's live SQLPage components.
CREATE TABLE customers (
    id INTEGER PRIMARY KEY,
    name TEXT NOT NULL,
    plan TEXT NOT NULL,
    status TEXT NOT NULL,
    seats INTEGER NOT NULL
);
INSERT INTO customers VALUES
    (1, 'Acme Corp', 'Pro', 'Active', 12),
    (2, 'Globex', 'Team', 'Active', 8),
    (3, 'Initech', 'Free', 'Trial', 3),
    (4, 'Umbrella', 'Pro', 'Late', 20);

CREATE TABLE tickets (
    id INTEGER PRIMARY KEY,
    customer_id INTEGER NOT NULL REFERENCES customers(id),
    resolved_at TEXT NOT NULL
);
WITH RECURSIVE ticket(id) AS (
    SELECT 1 UNION ALL SELECT id + 1 FROM ticket WHERE id < 42
)
INSERT INTO tickets
SELECT id, 1 + (id % 4),
    CASE WHEN id <= 18 THEN '2026-09-28 14:30:00'
         WHEN id <= 32 THEN '2026-09-29 10:15:00'
         ELSE '2026-09-30 16:45:00' END
FROM ticket;

CREATE TABLE customer_feedback (
    id INTEGER PRIMARY KEY,
    customer_id INTEGER NOT NULL REFERENCES customers(id),
    satisfied BOOLEAN NOT NULL
);
WITH RECURSIVE response(id) AS (
    SELECT 1 UNION ALL SELECT id + 1 FROM response WHERE id < 50
)
INSERT INTO customer_feedback
SELECT id, 1 + (id % 4), id <> 50 FROM response;

CREATE TABLE profiles (
    id INTEGER PRIMARY KEY,
    name TEXT NOT NULL,
    team TEXT NOT NULL
);
INSERT INTO profiles VALUES (1, 'Ada', 'SQL builders');
