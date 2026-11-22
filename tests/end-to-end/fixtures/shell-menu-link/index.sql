SELECT 'shell' AS component,
    'Demo' AS title,
    '{"title": "Reports", "link": "reports.sql"}' AS menu_item,
    '{"title": "More", "submenu": [{"title": "Settings", "link": "settings.sql"}]}' AS menu_item,
    'about' AS menu_item;
SELECT 'text' AS component, 'Menu demo' AS contents;
