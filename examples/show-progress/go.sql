-- can disable the spinner to show only progress bar
SELECT 'loader-start' AS component,  'non-existent-class' AS spinner;

SELECT 'progress' AS component,
  NULL AS percent,
  'sm' AS size,
  'yellow' AS color,
  'Working on it' AS stage;
SELECT sqlpage.exec('sleep', '3');

SELECT 'loader-stop' AS component,  NULL AS spinner;

SELECT 'text' AS component, 'Processing complete.' AS contents;
