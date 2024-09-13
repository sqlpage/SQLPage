-- =============================================================================
-- Checks for the availablity of an active session and redirects to the login
-- page, if necessary. Using a customized shell_ex component, shows "user" and
-- login/logout buttons appropriately in the top-right corner.
--
-- Note, any additonal "shell" component settings must also be included here in
-- the same component. It may require extending this script in a flexible way or
-- creating a page-specific copy, which is less desirable, as it would cause code
-- duplication.
--
-- ## Usage
--
-- Execute this script via
--
-- ```sql
-- SELECT
--     'dynamic' AS component,
--     sqlpage.run_sql('header_shell_session.sql') AS properties;
-- ```
--
-- at the top of the page script, but AFTER setting the required variables
--
-- ```sql
-- set _curpath = sqlpage.path();
-- set _session_required = 1;
-- set _shell_enabled = 1;
-- ```
--
-- ## Reuired SET Variables
--
-- $_curpath
--  - indicates redirect target passed to the login script
-- $_session_required
--  - 1 - require valid active session for non-public pages
--  - 0 - ignore active session for public pages
-- $_shell_enabled
--  - 1 - execute the shell component in this script (default, if not defined)
--  - 0 - do not execute the shell component in this script
--    Define this value to use page-specific shell component.
--    It id also necessary for no-GUI pages, which are called via a redirect and
--    normally redirect back after the necessary processing is completed. Such
--    pages may still require this script to check for active session, but they
--    will not be able to redirect back if this script outputs GUI buttons.

-- =============================================================================
-- ======================= Check required variables ============================
-- =============================================================================
--
-- Set default values (for now) for required variables.
-- Probably should instead show appropriate error messages and abort rendering.

set _curpath = ifnull($_curpath, '/');
set _session_required = ifnull($_session_required, 1);
set _shell_enabled = ifnull($_shell_enabled, 1);

-- =============================================================================
-- ========================= Check active session ==============================
-- =============================================================================
--
-- Check if session is available.
-- Require the user to log in again after 1 day

set _username = (
    SELECT username
    FROM sessions
    WHERE sqlpage.cookie('session_token') = id
      AND created_at > datetime('now', '-1 day')
);

-- Redirect to the login page if the user is not logged in.
-- Unprotected pages must
-- set _session_required = (SELECT FALSE);
-- before running this script

SELECT
    'redirect' AS component,
    '/login.sql?path=' || $_curpath AS link
WHERE $_username IS NULL AND $_session_required;

-- =============================================================================
-- ==================== Add User and Login/Logout buttons ======================
-- =============================================================================
--

SELECT
   'dynamic' AS component,
    json_array(
        json_object(
            'component',      'shell',
            'title',          'CRUD with Authentication',
            'icon',           'database',
            'description',    'Description',
            'layout',         'fluid',

            'css',
                json_array(
                    '/css/prism-tabler-theme.css', -- Load for code highlighting
                    '/css/style.css'
                ),

            'javascript',
                json_array(
                     
                     -- Code highlighting scripts

                    'https://cdn.jsdelivr.net/npm/prismjs@1/components/prism-core.min.js',
                    'https://cdn.jsdelivr.net/npm/prismjs@1/plugins/autoloader/prism-autoloader.min.js'
                ),

            'menu_item',
                iif($_username IS NOT NULL,
                    json_array(
                        json_object(
                            'button',  FALSE,
                            'title',   'Settings',
                            'icon',    'settings',
                            'submenu', json_array(
                                           json_object(
                                               'button',  TRUE,
                                               'title',   '',
                                               'icon',    'user-circle',
                                               'shape',   'pill',
                                               'size',    'sm',
                                               'narrow',  TRUE,
                                               'color',   'yellow',
                                               'outline', '',
                                               'link',    '#',
                                               'tooltip', 'User profile - Not Implemented'
                                           ),
                                           json_object(
                                               'button',  TRUE,
                                               'title',   '',
                                               'icon',    'logout',
                                               'shape',   'pill',
                                               'size',    'sm',
                                               'narrow',  TRUE,
                                               'color',   'green',
                                               'outline', '',
                                               'link',    '/logout.sql?path=' || $_curpath,
                                               'tooltip', 'Logout'
                                           )
                                       )
                        )
                    ),
                    json_array(
                        json_object(
                            'button',  TRUE,
                            'title',   '',
                            'icon',    'user-scan',
                            'shape',   'pill',
                            'size',    'sm',
                            'narrow',  TRUE,
                            'color',   'warning',
                            'outline', '',
                            'link',    '#',
                            'tooltip', 'Sign Up - Not Implemented'
                        ),
                       json_object(
                           'button',  TRUE,
                           'title',   '',
                           'icon',    'login',
                           'shape',   'pill',
                           'size',    'sm',
                           'narrow',  TRUE,
                           'color',   '',
                           'outline', 'cyan',
                           'link',    '/login.sql?path=' || $_curpath,
                           'tooltip', 'Login'
                       )
                    )
                )

        )
    ) AS properties
WHERE CAST($_shell_enabled AS INT) <> 0;
