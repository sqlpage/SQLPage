BEGIN;
CREATE TEMPORARY TABLE t(f INTEGER NOT NULL);
INSERT INTO t(f) VALUES ($x);
COMMIT;

select 'text' as component, f as contents from t;