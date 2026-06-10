-- Trusted page: loads a reserved (sqlpage/) file via the privileged run_sql,
-- which populates the shared sql_file_cache with the parsed private file.
select 'dynamic' as component,
    sqlpage.run_sql('sqlpage/private_cache_bypass_test.sql') as properties;
