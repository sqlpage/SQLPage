CREATE TABLE tap(
    tapping_session INTEGER,
    day REAL NOT NULL, -- fractional julian day, easy to manipulate in SQLite
    PRIMARY KEY (tapping_session, day)
);

CREATE VIEW tap_bpm AS
SELECT
    *,
    CAST(
        1 / ((24 * 60) * (day - previous))
        AS INTEGER
    ) AS bpm
FROM (
    SELECT 
        *,
        lag(day) OVER (ORDER BY day) AS previous
    FROM tap
);