SELECT 'spinner-start' AS component,
  -- default is "spinner-border"
  "spinner-grow text-red" AS spinner;
SELECT 'spinner-progress' AS component,
  '0' AS percent,
  'Sleeping 1 second' AS stage;
SELECT sqlpage.exec('sleep', '1');

SELECT 'spinner-progress' AS component,
  '20' AS percent,
  'Sleeping 1 more second' AS stage;
SELECT sqlpage.exec('sleep', '1');

/* stage property is optional */
SELECT 'spinner-progress' AS component,
  '40' AS percent;
SELECT sqlpage.exec('sleep', '1');

SELECT 'spinner-progress' AS component,
  '60' AS percent,
  'Sleeping 2 seconds' AS stage;
SELECT sqlpage.exec('sleep', '2');

SELECT 'spinner-progress' AS component,
  '100' AS percent;

SELECT 'spinner-stop' AS component;

SELECT 'text' AS component,
  'It works!' AS title,
  TRUE AS center,
  'Page is loaded.' AS contents;
