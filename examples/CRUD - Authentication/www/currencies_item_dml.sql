-- Procesess CREATE/UPDATE/DELETE operations.

-- =============================================================================
-- =========================== Module Setting ==================================
-- =========================== Login / Logout ==================================
-- =============================================================================

-- $_curpath and $_session_required are required for header_shell_session.sql.

SET $_session_required = 1;
SET $_shell_enabled = 0;

SELECT
    'dynamic' AS component,
    sqlpage.run_sql('header_shell_session.sql') AS properties;

-- =============================================================================
-- Redirect target must be passed as $path
-- =============================================================================

SET $_err_msg = '&path URL GET parameter (redirect target) is not set!';

SELECT
    'alert'          AS component,
    'red'            AS color,
    'alert-triangle' AS icon,
    'Error'          AS title,
    $_err_msg        AS description,
    TRUE             AS dismissible
WHERE
    $path IS NULL AND $DEBUG IS NULL;

-- =============================================================================
-- Check new values for validity:
--   - UPDATE existing record:
--       :id IS NOT NULL
--       If :name is in the database, :id must match
--       If attempting to change :name, operation may fail due to FK constraint
--   - INSERT new record:
--       :id IS NULL
--       :name is not in the database
-- =============================================================================

-- Pass new values back as JSON object in $values GET variable for form population.
--
-- For new records, the id (INTEGER PRIMARY KEY AUTOINCREMENT) should be set to NULL.
-- The id field is set as hidden in the record edit form and passed as the :id POST
-- variable. NULL, however, cannot be passed as such and is converted to blank string.
-- Check :id for '' and SET $id (:id will return the same value).

SET $_id = iif(typeof(:id) = 'text' AND :id = '', NULL, :id);

SET $_values = json_object(
    'id', CAST($_id AS INT),
    'name', :name,
    'to_rub', CAST(:to_rub AS NUMERIC)
);

SET $_op = iif($_id IS NULL, 'INSERT', 'UPDATE');
SET $_err_msg = sqlpage.url_encode('New currency already in the database');

SELECT
    'redirect' AS component,
    $path || '?' ||
      '&op='     || $_op     ||
      '&values=' || $_values ||
      '&error='  || $_err_msg AS link
FROM currencies
WHERE currencies.name = :name
  AND ($_id IS NULL OR currencies.id <> $_id);

-- =============================================================================
-- UPSERT: If everything is OK and "UPDATE" is indicated, update the database
-- =============================================================================

INSERT INTO currencies(id, name, to_rub)
    SELECT CAST($_id AS INT), :name, CAST(:to_rub AS NUMERIC)
    WHERE $action = 'UPDATE'
ON CONFLICT(id) DO
UPDATE SET name = excluded.name, to_rub = excluded.to_rub
RETURNING
    'redirect' AS component,
    $path || '?' ||
      '&id='   ||  id  ||
      '&info=' || $_op || ' completed successfully' AS link;

-- =============================================================================
-- DELETE
-- =============================================================================

DELETE FROM currencies
WHERE $action = 'DELETE' AND id = $_id
RETURNING
    'redirect' AS component,
    $path || '?' ||
      '&info=DELETE completed successfully' AS link;

-- =============================================================================
-- DEBUG
-- =============================================================================

SELECT
    'dynamic' AS component,
    sqlpage.run_sql('footer_debug_post-get-set.sql') AS properties;
