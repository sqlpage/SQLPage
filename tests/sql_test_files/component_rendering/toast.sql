SELECT 'toast' AS component, 'Default toast' AS title, 'It works !' AS description;
SELECT 'toast' AS component, 'Persistent' AS title, 'It works !' AS description, 0 AS duration, 1 AS dismissible;
SELECT 'toast' AS component, 'No close control' AS title, 'It works !' AS description, 0 AS duration, 0 AS dismissible;
SELECT 'toast' AS component, 'Colored' AS title, 'It works !' AS description, 'check' AS icon, 'green' AS color;
SELECT 'toast' AS component, '<strong>Escaped</strong> It works !' AS description;
SELECT 'toast' AS component, '**Markdown** [link](https://example.com) It works !' AS description_md;
SELECT 'toast' AS component, 'Alternate position' AS title, 'It works !' AS description, 'bottom-center' AS position;
SELECT 'toast' AS component, 'Hash-triggered' AS title, 'It works !' AS description, 'notice' AS id, 'notice' AS trigger;
