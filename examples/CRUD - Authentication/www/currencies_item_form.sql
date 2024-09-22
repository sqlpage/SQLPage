-- Reads an item from the database if valid id is provided and
-- populates the form. Otherwise, an empty form is presented.

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

set _getpath = '?path=' || ifnull($path, $_curpath);
set _action_target = 'currencies_item_dml.sql' || $_getpath;
set _table_list = 'currencies_list.sql';

-- =============================================================================
-- ========================== Filter invalid $id ===============================
-- =============================================================================
--
-- NULL is passed as 0 or '' via POST

SELECT
    'redirect' AS component,
    $_curpath AS link
WHERE $id = '' OR CAST($id AS INT) = 0;

-- If $id is set, it must be a valid PKEY value.

set error_msg = sqlpage.url_encode('Bad {id = ' || $id || '} provided');

SELECT
    'redirect' AS component,
    $_curpath || '?error=' || $error_msg AS link

-- If $id IS NULL, NOT IN returns NULL and redirect is NOT selected.

WHERE $id NOT IN (SELECT currencies.id FROM currencies);

-- =============================================================================
-- ======================== Filter invalid $values =============================
-- =============================================================================
--
-- If $values is provided, it must contain a valid JSON.

set _err_msg =
    sqlpage.url_encode('Values is set to bad JSON: __ ') || $values || ' __';

SELECT
    'redirect' AS component,
    $_curpath || '?error=' || $_err_msg AS link

-- Covers $values IS NULL..

WHERE NOT json_valid($values);

-- =============================================================================
-- ============================= Prepare data ==================================
-- =============================================================================
--
-- Field values may be provided via the $values GET variable formatted as JSON
-- object. If $values contains a valid JSON, use it to populate the form.
-- Otherwise, if $id is set to a valid value, retrieve the record from the
-- database and set values. If not, set values to all NULLs.

set _values = (
    WITH
        fields AS (
            -- If valid "id" is supplied as a GET variable, retrieve the record and
            -- populate the form.

            SELECT id, name, to_rub
            FROM currencies
            WHERE id = CAST($id AS INT) AND $values IS NULL

            -- If no "id" is supplied, the first part does not return any records,
            -- so add a dummy record.

                UNION ALL
            SELECT NULL, '@', 1
            WHERE $id IS NULL AND $values IS NULL

            -- If $value contains a valid JSON, use it to populate the form

                UNION ALL
            SELECT
                $values ->> '$.id' AS id,
                $values ->> '$.name' AS name,
                $values ->> '$.to_rub' AS to_rub
            WHERE json_valid($values)
        )
    SELECT
        json_object(
            'id',     CAST(fields.id AS INT),
            'name',   fields.name,
            'to_rub', CAST(CAST(fields.to_rub AS TEXT) AS NUMERIC)
        )
    FROM fields
);

-- =============================================================================
-- ========================= Browse Records Button =============================
-- =============================================================================
--
SELECT 
    'button'            AS component,
    'square'            AS shape,
    'sm'                AS size,
    'end'               AS justify;
SELECT                             
    'BROWSE'            AS title,
    'browse_rec'        AS id,
    'corner-down-left'  AS icon,
    'corner-down-left'  AS icon_after,
    'green'             AS outline,
    TRUE                AS narrow,
    $_table_list        AS link,
    'Browse full table' AS tooltip
WHERE NOT ifnull($action = 'DELETE', FALSE);

-- =============================================================================
-- ============================== Main Form ====================================
-- =============================================================================
--
-- When confirming record deletion, set all fields to read-only and id type to
-- number. No need to worry about the field values: all fields. including id are
-- passed back as POST variables, and the code above sets the $_values variable
-- for proper initialization of the reloaded form.

set _valid_ids = (
    SELECT json_group_array(
        json_object('label', CAST(id AS TEXT), 'value', id) ORDER BY id
    )
    FROM currencies
    WHERE ifnull($action, '') <> 'INSERT'
        UNION ALL
    SELECT '[]'
    WHERE $action = 'INSERT'
);
set _valid_ids = (
    json_insert($_valid_ids, '$[#]',
        json_object('label', 'NULL', 'value', json('null'))
    )
);

SELECT
    'dynamic' AS component,
    json_array(
        json_object(
            'component',      'form',
            'title',          'Currency',
            'class',          'form_class',
            'id',             'detail_view',
            'validate',       '',
            'action',         $_action_target
        ),
        json_object(
            'name',           'id',
            'label',          'ID',
            'type',           iif(ifnull($action = 'DELETE', FALSE), 'number', 'select'),
            'name',           'id',
            'value',          $_values ->> '$.id',
            'options',        $_valid_ids,
            'width',          4,
            'readonly',       ifnull($action = 'DELETE', FALSE),
            'required',       json('false')
        ),
        json_object(
            'name',           'name',
            'label',          'Currency',
            'value',          $_values ->> '$.name',
            'placeholder',    'RUR',
            'width',          4,
            'readonly',       ifnull($action = 'DELETE', FALSE),
            'required',       json('true')
        ),
        json_object(
            'type',           'number',
            'step',           0.01,
            'name',           'to_rub',
            'label',          'Exchange Rate to RUR',
            'value',          $_values ->> '$.to_rub',
            'placeholder',    1,
            'width',          4,
            'readonly',       ifnull($action = 'DELETE', FALSE),
            'required',       json('true')
        )
    ) AS properties
;

-- =============================================================================
-- ===================== Display DELETE confirmation ===========================
-- =============================================================================

SELECT
    'alert'                             AS component,
    'warning'                           AS color,
    'alert-triangle'                    AS icon,
    TRUE                                AS important,
    'Warning'                           AS title,
    'Confirm record deletion'           AS description
WHERE $action = 'DELETE';

-- =============================================================================
-- ========================== Main Form Buttons ================================
-- =============================================================================
--
-- When confirming record deletion, disable the UPDATE button, replace
-- the Reload button with the Cancel button, invert the DELETE button by
-- removing the outline color, and ajust the POST target.


SELECT 
    'button'                            AS component,
    'pill'                              AS shape,
    ''                                  AS size,
    'center'                            AS justify;

SELECT                                                  -- Default button
    '(Re)load'                          AS title,
    'read_rec'                          AS id,
    'database'                          AS icon,
    'database'                          AS icon_after,
    'green'                             AS outline,
    TRUE                                AS narrow, 
    $_curpath                           AS link,
    'detail_view'                       AS form,
    TRUE                                AS space_after
WHERE NOT ifnull($action = 'DELETE', FALSE);

SELECT                                                  -- Cancel DELETE button
    'Cancel'                            AS title,
    'read_rec'                          AS id,
    'alert-triangle'                    AS icon,
    'alert-triangle'                    AS icon_after,
    'primary'                           AS color,
    TRUE                                AS narrow, 
    $_curpath                           AS link,
    'detail_view'                       AS form,
    TRUE                                AS space_after
WHERE ifnull($action = 'DELETE', FALSE);

SELECT                         
    'Update'                            AS title,       -- UPDATE / INSERT button 
    'update_rec'                        AS id,
    'device-floppy'                     AS icon,
    'device-floppy'                     AS icon_after,
    'azure'                             AS outline,
    TRUE                                AS narrow, 
    $_action_target || '&action=UPDATE' AS link,
    'detail_view'                       AS form,
    ifnull($action = 'DELETE', FALSE)   AS disabled,
    TRUE                                AS space_after;

SELECT                                                  -- DELETE button
    'DELETE'                            AS title,
    'delete_rec'                        AS id,
    'alert-triangle'                    AS icon,
    'trash'                             AS icon_after,
    TRUE                                AS narrow, 
    iif(ifnull($action = 'DELETE', FALSE), NULL, 'danger') AS outline,
    iif(ifnull($action = 'DELETE', FALSE),
        $_action_target, $_curpath || '?') || '&action=DELETE' AS link,
    'danger'                            AS color,
    'detail_view'                       AS form,
    FALSE                               AS space_after;

-- =============================================================================
-- ======================== Display confirmation ===============================
-- =============================================================================

SELECT
    'alert'          AS component,
    'green'          AS color,
    'check'          AS icon,
    'Success'        AS title,
    $info            AS description,
    True             AS dismissible
WHERE $info IS NOT NULL;

-- =============================================================================
-- ======================== Display error message ==============================
-- =============================================================================

SELECT
    'alert'          AS component,
    'red'            AS color,
    'thumb-down'     AS icon,
    $op || ' error'  AS title,
    $error           AS description,
    True             AS dismissible
WHERE $error IS NOT NULL;

-- =============================================================================
-- DEBUG
-- =============================================================================

SELECT
    'dynamic' AS component,
    sqlpage.run_sql('footer_debug_post-get-set.sql') AS properties;
