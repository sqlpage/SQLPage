SET "First name" = 'Ada';
SELECT :"First name" AS actual, 'Ada' AS expected;
SELECT LOWER(:"First name") AS actual, 'ada' AS expected;
SELECT sqlpage.url_encode(:"First name") AS actual, 'Ada' AS expected;
SELECT :"Missing name" AS actual, NULL AS expected;
