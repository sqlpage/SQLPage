SELECT 'hero' as component,
    'SQLPage Form Demo' as title,
    'This application allows you to manage a list of users and their addresses' as description_md,
    'https://upload.wikimedia.org/wikipedia/commons/thumb/e/e4/Lac_de_Zoug.jpg/640px-Lac_de_Zoug.jpg' as image,
    'edit_user.sql' as link,
    'Create a new user' as link_text;

SELECT 'list' AS component, 'Users' AS title;
SELECT first_name || ' ' || last_name AS title, email AS description, 'edit_user.sql?id=' || id AS link FROM "user";
SELECT 'Add a new user' AS title, 'edit_user.sql' AS link, TRUE AS active;