START TRANSACTION; -- mysql syntax
CREATE TEMPORARY TABLE IF NOT EXISTS t(f VARCHAR(255) NOT NULL); -- mysql does not remove temporary tables on rollback
INSERT INTO t(f) SELECT 'bad value' where $x is null;
SELECT 'debug' as component, f from t;
INSERT INTO t(f) VALUES ($x); -- this will fail if $x is null. And the transaction should be rolled back, so 'bad value' should not be in the table
COMMIT;

select 'text' as component, $x as contents
where not exists (select 1 from t where f = 'bad value');

