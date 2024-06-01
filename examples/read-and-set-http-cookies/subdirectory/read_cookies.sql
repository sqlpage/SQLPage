-- Cookies can be specific to a certain path in the website
-- This page demonstrates that a gobal cookie can be removed from a subdirectory
select 'cookie' as component, 'username' as name, true as remove, '/' as path;

SELECT 'text' as component;
SELECT 'The value of your username cookie was: ' ||
    COALESCE(sqlpage.cookie('username'), 'NULL') ||
    '. It has now been removed. You can reload this page.' as contents;

