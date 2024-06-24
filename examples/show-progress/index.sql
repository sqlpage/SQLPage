SELECT 'shell' AS component,
  'dark' AS theme;

SELECT 'loader-start' AS component,
  -- default is "spinner-border"
  "spinner-grow text-red" AS spinner;

SELECT 'progress' AS component,
  'sm' AS size,
  'yellow' AS color,
  '0' AS percent,
  'Sleeping 1 second' AS stage;
SELECT sqlpage.exec('sleep', '1');

/* percent property is optional */
SELECT 'progress' AS component,
  NULL AS percent,
  'sm' AS size,
  'yellow' AS color,
  'Doing something' AS stage;
SELECT sqlpage.exec('sleep', '1');

/* stage property is optional */
SELECT 'progress' AS component,
  'sm' AS size,
  'yellow' AS color,
  '40' AS percent;
SELECT sqlpage.exec('sleep', '1');

/* multiple rows */
SELECT 'progress' AS component, 'sm' AS size, 'yellow' AS color;
SELECT '70' AS percent, 'Sleeping 1 second' AS stage
SELECT sqlpage.exec('sleep', '1');
SELECT '80' AS percent, 'Sleeping 1 more second' AS stage;
SELECT sqlpage.exec('sleep', '1');
SELECT '90' AS percent, 'Sleeping 1 second again' AS stage;
SELECT sqlpage.exec('sleep', '1');

SELECT '100' AS percent;

SELECT 'loader-stop' AS component;

SELECT 'text' AS component,
  'It works!' AS title,
  TRUE AS center,
  'Page is loaded.' AS contents;

SELECT 'button' AS component;
SELECT 'Go' AS title, '/go.sql' AS link;

-- can use progress on it's own
SELECT 'progress' AS component, 'Waiting for user' AS stage;
