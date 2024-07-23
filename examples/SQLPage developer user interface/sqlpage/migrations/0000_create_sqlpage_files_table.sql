CREATE TABLE
  sqlpage_files (
    path VARCHAR(255) NOT NULL PRIMARY KEY,
    contents BYTEA NOT NULL,
    last_modified TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
  );

-- automatically update last_modified timestamp

CREATE OR REPLACE FUNCTION update_last_modified_sqlpage_files()
RETURNS TRIGGER AS $$
BEGIN
  NEW.last_modified = CURRENT_TIMESTAMP;
  RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER update_last_modified BEFORE
UPDATE ON sqlpage_files FOR EACH ROW
EXECUTE PROCEDURE update_last_modified_sqlpage_files();