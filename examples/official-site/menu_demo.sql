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
                "title": "",
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
                        "color": "yellow"
                    },
                    {
                        "link": "/performance.sql",
                        "icon": "logout",
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
                "title": "Examples",
                "icon": "trash",
                "submenu": [
                    {
                        "link": "/examples/tabs.sql",
                        "icon": "device-floppy",
                        "title": "Tabs"
                    },
                    {
                        "link": "/examples/layouts.sql",
                        "button": true,
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
                        "link": "//github.com/lovasoa/sqlpage/issues",
                        "title": "Report a bug"
                    }
                ]
            }
        ]
    }
]' AS properties;