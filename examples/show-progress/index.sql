SELECT 'spinner-start' AS component;
SELECT 'spinner-progress' AS component,
  '0' AS percent,
  'Sleeping 3 seconds' AS stage; 
SELECT sqlpage.exec('sleep', '3');

SELECT 'spinner-progress' AS component,
  '30' AS percent,
  'Sleeping 5 seconds' AS stage; 
SELECT sqlpage.exec('sleep', '5');

/* stage property is optional */
SELECT 'spinner-progress' AS component,
  '80' AS percent;
SELECT sqlpage.exec('sleep', '1');

SELECT 'spinner-stop' AS component;
