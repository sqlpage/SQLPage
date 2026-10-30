SELECT 'modal' AS component, 'nested-modal' AS id, 'Nested widgets' AS title WHERE $notification IS NULL;
SELECT 'toast' AS component, 'nested-toast' AS id, 'Nested notification' AS title, 0 AS duration WHERE $notification IS NOT NULL;
