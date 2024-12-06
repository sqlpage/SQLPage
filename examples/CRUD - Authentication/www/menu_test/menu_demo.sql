select 'dynamic' as component,
'[
    {
        "component": "shell",
        "title": "SQLPage",
        "icon": "database",
        "link": "/",
        "menu_item": [
            {
                "icon": "settings",
                "title": "Z",
                "button": true,
                "shape": "pill",
                "narrow": true,
                "outline": "warning",
                "submenu": [
                    {
                        "link": "/safety.sql",
                        "icon": "user",
                        "button": true,
                        "shape": "pill",
                        "size": "sm",
                        "tooltip": "User",
                        "color": "yellow"
                    },
                    {},
                    {
                        "link": "/performance.sql",
                        "icon": "logout",
                        "tooltip": "Logout",
                        "button": true,
                        "shape": "pill",
                        "size": "sm",
                        "outline": "warning"

                    },
                    {
                        "link": "/performance.sql",
                        "icon": "",
                        "tooltip": "Logout",
                        "button": true,
                        "shape": "pill",
                        "size": "sm",
                        "outline": "warning"

                    }

                ]
            },
            {
                "icon": "database",
                "title": "Dummy",
                "button": true,
                "shape": "pill",
                "narrow": true,
                "color": "green"
            },
            {
                "icon": "",
                "title": "",
                "button": true,
                "shape": "pill",
                "narrow": true,
                "color": "blue"
            },
            {
                "title": "Examples",
                "icon": "trash",
                "submenu": [
                    {
                        "link": "/examples/tabs/",
                        "icon": "device-floppy",
                        "title": "Tabs"
                    },
                    {
                        "link": "/examples/layouts.sql",
                        "button": true,
                        "tooltip": "Layouts",
                        "title": "Layouts",
                        "color": "primary"
                    },
                    {
                        "link": "/examples/multistep-form",
                        "title": "Forms"
                    }
                ]
            },
            {
                "title": "Community",
                "submenu": [
                    {
                        "link": "blog.sql",
                        "title": "Blog"
                    },
                    {
                        "link": "//github.com/sqlpage/SQLPage/issues",
                        "title": "Report a bug"
                    }
                ]
            }
        ]
    }
]' AS properties;

SELECT 
    'button'            AS component,
    'pill'              AS shape,
    ''                  AS size,
    'center'            AS justify;
SELECT                             
    ''            AS title,
    'browse_rec'        AS id,
    'green'             AS outline,
    TRUE                AS narrow,
    '#'                 AS link,
    '/icons/earth-icon.svg' AS img;
