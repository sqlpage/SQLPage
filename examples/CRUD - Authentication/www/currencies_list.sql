-- =============================================================================
-- =========================== Module Setting ==================================
-- =========================== Login / Logout ==================================
-- =============================================================================

-- $_curpath and $_session_required are required for header_shell_session.sql.

set _curpath = sqlpage.path();
set _session_required = 1;

SELECT
    'dynamic' AS component,
    sqlpage.run_sql('header_shell_session.sql') AS properties;

-- =============================================================================
-- =============================== Module vars =================================
-- =============================================================================

set _getpath = '&path=' || $_curpath;
set _item_form = 'currencies_item_form.sql';

-- =============================================================================
-- ======================== Display confirmation ===============================
-- =============================================================================

SELECT
    'alert'         AS component,
    'green'         AS color,
    'check'         AS icon,
    'Success'       AS title,
    $info           AS description,
    TRUE            AS dismissible
WHERE $info IS NOT NULL;

-- =============================================================================
-- ======================== Display error message ==============================
-- =============================================================================

SELECT
    'alert'         AS component,
    'red'           AS color,
    'thumb-down'    AS icon,
    $op || ' error' AS title,
    $error          AS description,
    TRUE            AS dismissible
WHERE $error IS NOT NULL;

-- =============================================================================
-- ========================== New record button ================================
-- =============================================================================

SELECT 
    'button'        AS component,
    'pill'          AS shape,
    'lg'            AS size,
    'end'           AS justify;
SELECT
    'New Record'    AS title,
    'insert_rec'    AS id,
    'circle-plus'   AS icon,
    'circle-plus'   AS icon_after,
    'green'         AS outline,
    $_item_form || '?' || $_getpath || '&action=INSERT' AS link
;

-- =============================================================================
-- ============================= Show the table ================================
-- =============================================================================

SELECT
    'divider'       AS component,
    'currencies'    AS contents;

-- =============================================================================

SELECT
    'title'         AS component,
    'Currencies'    AS contents,
    4               AS level,
    TRUE            AS center,
    'title_class'   AS class,
    'title_id'      AS id;

-- =============================================================================
-- TABLE

SELECT
    'table'         AS component,
    TRUE            AS sort,
    TRUE            AS search,
    TRUE            AS border,
    TRUE            AS hover,
    TRUE            AS striped_columns,
    TRUE            AS striped_rows,
    'table_class'   AS class,
    'table_id'      AS id,
    'actions'       AS markdown;

SELECT
    id,
    name,
    to_rub,
    '[![](/icons/outline/edit.svg)]('  || $_item_form || '?' || $_getpath || '&id=' || id || ') ' ||
    '[![](/icons/outline/trash.svg)](' || $_item_form || '?' || $_getpath || '&id=' || id || '&action=DELETE)' AS actions
FROM currencies
ORDER BY id;

-- =============================================================================
-- DEBUG
-- =============================================================================

SELECT
    'dynamic' AS component,
    sqlpage.run_sql('footer_debug_post-get-set.sql') AS properties;
