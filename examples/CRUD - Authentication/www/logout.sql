SELECT
    'cookie' AS component,
    'session_token' AS name,
    TRUE AS remove;

SELECT
    'redirect' AS component,
    iif(
        $path IS NOT NULL
        AND length($path) > 0
        AND (
            (substr($path, 1, 1) = '/' AND substr($path, 2, 1) <> '/')
            OR (
                (substr($path, 1, 1) GLOB '[A-Za-z0-9_]'
                    OR substr($path, 1, 2) = './'
                    OR substr($path, 1, 3) = '../')
                AND instr($path, ':') = 0
            )
        )
        AND instr($path, char(92)) = 0
        AND instr($path, '%') = 0
        AND instr($path, char(0)) = 0
        AND ($path GLOB '*[' || char(1) || '-' || char(31) || char(127) || ']*') = 0,
        $path,
        '/login.sql'
    ) AS link; -- redirect to the login page after logout when the path is unsafe
