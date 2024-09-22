-- set _locale_code = $lang;                         -- 'en', 'ru', 'de', 'fr', 'zh-cn'
-- set _theme = $theme;                              -- 'default', 'fancy'
-- set _hide_language_names = $hide_language_names;  -- 0, 1 (BOOLEAN)
-- set _authenticated = $authenticated;              -- 0, 1 (BOOLEAN)

-- =============================================================================
-- =============================================================================

WITH

    test_values(_locale_code,  _theme,   _hide_language_names, _authenticated ) AS (VALUES
--             (    'en',      'fancy',          TRUE,               FALSE)
--             (    NULL,        NULL,           NULL,               NULL)
               (    'fr',      'default',        TRUE,               TRUE)
    ),
     
    -- Replaces values with appropriate variables
    -- ifnull guuards from invalid JSON errors

    config_user AS (
        SELECT
            lower(ifnull(_locale_code, '_'))               AS locale_code,
            lower(ifnull(_theme, '_'))                     AS theme,
            CAST(_hide_language_names AS INTEGER)          AS hide_language_names,
            CAST(ifnull(_authenticated, FALSE) AS INTEGER) AS authenticated
        FROM test_values
    ),
    
    -- Inputs data guards
    
    config_guards AS (
        SELECT
            (
                SELECT iif(contents  ->> ('$.' || locale_code) IS NULL, NULL, locale_code)
                FROM sqlpage_files
                WHERE path = 'locales/locales.json'
            ) AS locale_code,
            (
                SELECT iif(contents  ->> ('$.' || theme) IS NULL, NULL, theme)
                FROM sqlpage_files
                WHERE path = 'themes/themes.json'
            ) AS theme,
            hide_language_names,
            authenticated
        FROM config_user
    ),
    
    -- Retrieves locale and theme JSON data
    
    config AS (
        SELECT
            iif(locale_code IS NULL, NULL, (
                SELECT contents ->> '$.menu'
                FROM sqlpage_files
                WHERE path = 'locales/' || locale_code || '/locale.json'
            )) AS locale,
            iif(theme IS NULL, NULL, (
                SELECT contents ->> '$.menu'
                FROM sqlpage_files
                WHERE path = 'themes/' || theme || '/theme.json'
            )) AS theme,
            (
                SELECT contents ->> '$.menu'
                FROM sqlpage_files
                WHERE path = 'themes/default/theme.json'
            ) AS theme_default,
            (
                SELECT contents ->> '$.meta.label'
                FROM sqlpage_files
                WHERE path = 'locales/' || locale_code || '/locale.json'
            ) AS locale_label,
            hide_language_names,
            authenticated
        FROM config_guards
    ),
    
    -- Prepares language items.
    -- This is a dynamically generated menu item.
    
    locale_codes AS (
        SELECT "key" AS code, value AS position
        FROM sqlpage_files, json_each(contents)
        WHERE sqlpage_files.path = 'locales/locales.json'
    ),
    languages AS (
        SELECT
            position,
            code,
            contents ->> '$.meta.label' AS label
        FROM sqlpage_files, locale_codes 
        WHERE path like 'locales/%/locale.json'
          AND contents ->> '$.meta.code' = code COLLATE NOCASE
        ORDER BY position
    ),
    
    -- Prepares a list of top menu items with default icons.
    -- The language menu includes the "global/neutral/undefinded" icon.
    
    top_menus_global AS (
        SELECT
            position,
            label,

            -- Hide top menu with submenu if a particular filter is included in
            -- state_filter and its current value does match the specified value.
            --
            -- Note that the "class" attribute is set to two classes:
            -- 'menu_' || lower(label) and the same with '_slim' suffix.
            -- These classes are applied to respective submenus for css-based fine-tuning.
            
            CASE
                WHEN (state_filter ->> '$.authenticated') IS NULL
                  OR state_filter ->> '$.authenticated' = ifnull(authenticated, FALSE) THEN
                    json_object(
                        'title', iif(icon_only IS TRUE, NULL, ifnull(locale ->> ('$.' || label), label)),
                        iif(theme IS NULL, 'icon', 'image'),
                        iif(theme IS NULL, tabler_icon, '/' || (theme ->> ('$.' || label))),
                        'class',
                                 -- Handles special cases: only sets the '_slim' class on the 'Language'
                                 -- menu when language names (labels) are hidden. Only set the base class
								 -- for English localization (the extra whitespaces are painfull...)
                                 iif(locale_label = 'English', ' menu_' || lower(label), '') ||								 
                                 iif(label = 'Language' AND hide_language_names IS NOT TRUE, '',
                                     ' menu_' || lower(label) || '_slim'
                                 )
                    )
                ELSE
                    NULL
            END AS top_item
        FROM menus, config
        WHERE parent_label IS NULL
        ORDER BY position
    ),

    -- The sole purpose of this separate modification is to set the icon
    -- of the top language menu to reflect the current locale

    top_menus AS (
        SELECT
            position,
            label,
            iif(label <> 'Language' OR locale IS NULL OR top_item IS NULL,
                top_item,
                json_set(
                    top_item,
                    '$.image',
                    '/' || (theme ->> ('$.' || locale_label))
                )
             ) AS top_item
        FROM top_menus_global, config
        ORDER BY position
    ),
    
    -- Prepares a list of submenu lines.
    
    menu_lines AS (
        SELECT
            parent_label,
            position,

            -- Hides menu line if a particular filter is included in state_filter
            -- and its current value does match the specified value
            
            CASE
                WHEN (state_filter ->> '$.authenticated') IS NULL
                  OR state_filter ->> '$.authenticated' = ifnull(authenticated, FALSE) THEN
                    json_object(
                        'title', iif(icon_only IS TRUE, NULL, ifnull(locale ->> ('$.' || label), label)),
                        iif(theme IS NULL, 'icon', 'image'),
                        iif(theme IS NULL, tabler_icon, '/' || (theme ->> ('$.' || label))),
                        'link', link
                    )
            END AS menu_line
        FROM menus, config
        WHERE parent_label IS NOT NULL

        -- Generates and appends the Language submenu lines
        
            UNION ALL
        SELECT
            'Language' AS parent_label,
            position,
            json_object(
                'title',
                iif(hide_language_names IS TRUE, NULL, ifnull(locale ->> ('$.' || label), label)),
                'image', '/' || (iif(theme IS NULL, theme_default, theme) ->> ('$.' || label)),
                'link', '/locales/locale.sql?lang=' || code
            ) AS menu_line
        FROM languages, config
        ORDER BY parent_label, position
    ),
    
    -- Groups menu lines into submenus
    
    menu_lists AS (
        SELECT
            parent_label,
            json_group_array(
                json(menu_line) ORDER BY position
            ) AS menu_list
        FROM menu_lines 
        GROUP BY parent_label
    ),
    
    -- Combines submenus with corresponding top menu lines yielding complete menu_item objects
    
    menu_sets AS (
        SELECT
            position,
            label,
            json_set(json(top_item), '$.submenu', json(menu_list)) AS menu_set
        FROM top_menus, menu_lists
        WHERE top_menus.label = menu_lists.parent_label
        ORDER BY position
    ),
    
    -- Prepares final array of menu_item objects to be used with the "dynamic" component
    
    menu AS (
        SELECT
            json_group_array(json(menu_set) ORDER BY position) AS menu
        FROM menu_sets
    ),

    -- shell_dynamic_static is included for debugging purposes. Call
    -- it to generate "static" SQL for inclusion in an SQLPage script.
    
    shell_dynamic_static AS (
        SELECT
            'SELECT' || x'0A' || '    ''dynamic'' AS component,' || x'0A' ||
            quote(json_pretty(json_object(
                'component', 'shell',
                'title', 'SQLPage',
                'icon', 'database',
                'link', '/',
                'css', '/css/style.css',
                'menu_item', json(menu)
            ))) || ' AS properties' || x'0A0A' AS properties
        FROM menu
    ),
    
    -- Call shell_dynamic if this script is processed directly by SQLPage.
    -- After copy-pasting adjust the input controls in the first CTE.
    
    shell_dynamic AS (
        SELECT
            'dynamic' AS component,
            json_object(
                'component', 'shell',
                'title', 'SQLPage',
                'icon', 'database',
                'link', '/',
                'css', '/css/style.css',
                'menu_item', json(menu)
            ) AS properties
        FROM menu
    )
SELECT * FROM shell_dynamic_static;
