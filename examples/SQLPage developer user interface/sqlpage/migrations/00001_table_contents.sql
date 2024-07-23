-- Given a table name as text, return the contents of the table as a set of json objects
-- Safely escapes the table name to prevent SQL injection.
-- Accepts only normal tables, not postgres system tables.
CREATE OR REPLACE FUNCTION table_contents (table_name text)
RETURNS SETOF json AS $$
BEGIN
  RETURN QUERY EXECUTE
    format('SELECT row_to_json(%I) FROM %I', table_name, table_name);
END;
$$ LANGUAGE plpgsql;