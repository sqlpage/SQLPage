DROP TABLE IF EXISTS _sqlx_migrations;

CREATE TABLE IF NOT EXISTS _sqlx_migrations (
    version         INTEGER PRIMARY KEY,
    description     TEXT COLLATE NOCASE NOT NULL,
    installed_on    TEXT COLLATE NOCASE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    success         INTEGER NOT NULL,
    checksum        BLOB    NOT NULL,
    execution_time  INTEGER NOT NULL
);


-- The path field should be relative to the www root. Do not
-- include absolute paths pointing to files outside the www root.

CREATE TABLE IF NOT EXISTS sqlpage_files (
    path          TEXT COLLATE NOCASE NOT NULL UNIQUE
        GENERATED ALWAYS AS (
            iif(prefix IS NOT NULL AND length(prefix) > 0, prefix || '/', '') ||
            name
        ),
    contents      BLOB,
    last_modified TEXT DEFAULT CURRENT_TIMESTAMP,
    prefix        TEXT COLLATE NOCASE NOT NULL DEFAULT '',
    name          TEXT COLLATE NOCASE NOT NULL,
    tag           TEXT COLLATE NOCASE,
    src_url       TEXT COLLATE NOCASE,
    PRIMARY KEY(prefix, name)
);


CREATE TRIGGER IF NOT EXISTS sqlpage_files_update
    AFTER UPDATE OF path, contents ON sqlpage_files
	WHEN old.last_modified = new.last_modified
    BEGIN
        UPDATE sqlpage_files
        SET last_modified = CURRENT_TIMESTAMP
		WHERE last_modified = new.last_modified;
    END;
