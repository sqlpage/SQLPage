select 'csv' as component;
-- Error before any data row: the CSV header is never written, so columns is
-- empty when handle_error runs.
select * from definitely_missing_table_xyz;
