-- =============================================================================
-- =========================== Module Setting ==================================
-- =========================== Login / Logout ==================================
-- =============================================================================

-- $_curpath and $_session_required are required for header_shell_session.sql.

SET $_curpath = sqlpage.path();
SET $_session_required = 0;

SELECT
    'dynamic' AS component,
    sqlpage.run_sql('header_shell_session.sql') AS properties;

-- =============================================================================

SELECT
    'text'  AS component,
    TRUE    AS center,
    2       AS level,
    'SQLite Introspection Information' AS title;

SELECT
    'divider'        AS component,
    'Password Hash'  AS contents;

-- =============================================================================
-- Password Hash

SELECT 
    'alert'          AS component,
    'green'          AS color,
    'edit'           AS icon,
    'Password Hash: sqlpage.hash_password(''admin'')' AS title,
    sqlpage.hash_password('admin') AS description,
    TRUE             AS dismissible;

-- =============================================================================
-- ============================ Alert Template =================================
-- =============================================================================

SELECT
    'divider'        AS component,
    'Alert Template' AS contents;

-- =============================================================================
-- ALERT

SELECT 
    'alert'              AS component,
    'green'              AS color,
    'Alert Title'        AS title,
    'Description'        AS description,
    '**Bold MD**'        AS description_md,
    'alert_class'        AS class,
    'alert_id'           AS id,
    TRUE                 AS dismissible,
    FALSE                AS important,
    'user'               AS icon,
    'https://google.com' AS link,
    'LINK TEXT'          AS link_text;

-- =============================================================================
-- ============================ IDs ============================================
-- =============================================================================

SELECT
    'divider'        AS component,
    'IDs'            AS contents;

-- =============================================================================

SELECT
    'title'          AS component,
    'IDs'            AS contents,
    4                AS level,
    TRUE             AS center,
    'title_class'    AS class,
    'title_id'       AS id;

-- =============================================================================
-- TABLE

SELECT
    'table'       AS component,
    TRUE          AS sort,
    TRUE          AS search,
    TRUE          AS border,
    TRUE          AS hover,
    TRUE          AS striped_columns,
    TRUE          AS striped_rows,
    'table_class' AS class,
    'table_id'    AS id;

SELECT
    sqlite_version() AS "SQLite Version",
    (SELECT * FROM pragma_application_id()) AS app_id,
    (SELECT * FROM pragma_user_version())   AS user_version,
    (SELECT * FROM pragma_schema_version()) AS schema_version;

-- =============================================================================
-- ============================ SQLite_Master ==================================
-- =============================================================================

SELECT
    'divider'        AS component,
    'SQLite_Master'  AS contents;

-- =============================================================================

SELECT
    'title'          AS component,
    'SQLite_Master'  AS contents,
    4                AS level,
    TRUE             AS center,
    'title_class'    AS class,
    'title_id'       AS id;

-- =============================================================================
-- TABLE

SELECT
    'table'                AS component,
    TRUE                   AS sort,
    TRUE                   AS search,
    TRUE                   AS border,
    TRUE                   AS hover,
    TRUE                   AS striped_columns,
    TRUE                   AS striped_rows,
    TRUE                   AS small,
    'table_class'          AS class,
    TRUE                   AS overflow;

SELECT * FROM sqlite_master;

-- =============================================================================
-- ============================ Function List ==================================
-- =============================================================================

SELECT
    'divider'        AS component,
    'Function List'  AS contents;

-- =============================================================================

SELECT
    'Function List. Total ' || (SELECT count(DISTINCT name)
                                FROM pragma_function_list())
                            || ' distinct' AS contents,
    'title'          AS component,
    4                AS level,
    TRUE             AS center,
    'title_class'    AS class,
    'title_id'       AS id;

-- =============================================================================
-- TABLE

SELECT
    'table'       AS component,
    TRUE          AS sort,
    TRUE          AS search,
    TRUE          AS border,
    TRUE          AS hover,
    TRUE          AS striped_columns,
    TRUE          AS striped_rows,
    TRUE          AS small,
    'table_class' AS class,
    'table_id'    AS id;

SELECT * FROM pragma_function_list() ORDER BY name, narg;

-- =============================================================================
-- ============================ Collation List =================================
-- =============================================================================

SELECT
    'divider'        AS component,
    'Collation List' AS contents;

-- =============================================================================

SELECT
    'Collation List' AS contents,
    'title'          AS component,
    4                AS level,
    TRUE             AS center,
    'title_class'    AS class,
    'title_id'       AS id;

-- =============================================================================
-- TABLE

SELECT
    'table'       AS component,
    TRUE          AS sort,
    TRUE          AS search,
    TRUE          AS border,
    TRUE          AS hover,
    TRUE          AS striped_columns,
    TRUE          AS striped_rows,
    TRUE          AS small,
    'table_class' AS class,
    'table_id'    AS id;

SELECT * FROM pragma_collation_list() ORDER BY rowid;

-- =============================================================================
-- ============================ Pragma List ====================================
-- =============================================================================

SELECT
    'divider'        AS component,
    'Pragma List'    AS contents;

-- =============================================================================

SELECT
    'Pragma List. Total ' || (SELECT count(*) FROM pragma_pragma_list()) AS contents,
    'title'          AS component,
    4                AS level,
    TRUE             AS center,
    'title_class'    AS class,
    'title_id'       AS id;

-- =============================================================================
-- TABLE

SELECT
    'table'       AS component,
    TRUE          AS sort,
    TRUE          AS search,
    TRUE          AS border,
    TRUE          AS hover,
    TRUE          AS striped_columns,
    TRUE          AS striped_rows,
    TRUE          AS small,
    'table_class' AS class,
    'table_id'    AS id;

SELECT * FROM pragma_pragma_list() AS functions ORDER BY name;

-- =============================================================================
-- ============================ Module List ====================================
-- =============================================================================

SELECT
    'divider'        AS component,
    'Pragma List'    AS contents;

-- =============================================================================

SELECT
    'Module List. Total ' || (SELECT count(*) FROM pragma_module_list()) AS contents,
    'title'          AS component,
    4                AS level,
    TRUE             AS center,
    'title_class'    AS class,
    'title_id'       AS id;

-- =============================================================================
-- TABLE

SELECT
    'table'       AS component,
    TRUE          AS sort,
    TRUE          AS search,
    TRUE          AS border,
    TRUE          AS hover,
    TRUE          AS striped_columns,
    TRUE          AS striped_rows,
    TRUE          AS small,
    'table_class' AS class,
    'table_id'    AS id;

SELECT * FROM pragma_module_list() ORDER BY name;

-- =============================================================================
-- ============================ Table List =====================================
-- =============================================================================

SELECT
    'divider'        AS component,
    'Table List'     AS contents;

-- =============================================================================

SELECT
    'Table List'     AS contents,
    'title'          AS component,
    4                AS level,
    TRUE             AS center,
    'title_class'    AS class,
    'title_id'       AS id;

-- =============================================================================
-- TABLE

SELECT
    'table'       AS component,
    TRUE          AS sort,
    TRUE          AS search,
    TRUE          AS border,
    TRUE          AS hover,
    TRUE          AS striped_columns,
    TRUE          AS striped_rows,
    TRUE          AS small,
    'table_class' AS class,
    'table_id'    AS id;

SELECT * FROM pragma_table_list() ORDER BY type, name;

-- =============================================================================
-- ============================ Database List ==================================
-- =============================================================================

SELECT
    'divider'        AS component,
    'Database List'  AS contents;

-- =============================================================================

SELECT
    'Database List'  AS contents,
    'title'          AS component,
    4                AS level,
    TRUE             AS center,
    'title_class'    AS class,
    'title_id'       AS id;

-- =============================================================================
-- TABLE

SELECT
    'table'       AS component,
    TRUE          AS sort,
    TRUE          AS search,
    TRUE          AS border,
    TRUE          AS hover,
    TRUE          AS striped_columns,
    TRUE          AS striped_rows,
    TRUE          AS small,
    'table_class' AS class,
    'table_id'    AS id;

SELECT * FROM pragma_database_list() ORDER BY seq;

-- =============================================================================
-- DEBUG
-- =============================================================================

SELECT
    'dynamic' AS component,
    sqlpage.run_sql('footer_debug_post-get-set.sql') AS properties;
