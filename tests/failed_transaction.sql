BEGIN;
CREATE TEMPORARY TABLE t(f VARCHAR(100) NOT NULL);
INSERT INTO t(f) VALUES ($x);
COMMIT;

select 'text' as component, f as contents from t;