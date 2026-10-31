SELECT 'shell' AS component;

-- Opens once on load: green header, markdown body, footer close label.
SELECT 'modal' AS component,
    'notice' AS id,
    'Saved' AS title,
    'green' AS color,
    TRUE AS visible,
    'Close' AS close
WHERE COALESCE($scenario, 'open') = 'open';

SELECT 'It works !' AS contents_md
WHERE COALESCE($scenario, 'open') = 'open';

-- Stays closed until #notice. `visible` is omitted, or false when requested.
SELECT 'modal' AS component,
    'notice' AS id,
    'Saved' AS title,
    'green' AS color
WHERE $scenario = 'closed' AND $visible IS NULL;

SELECT 'modal' AS component,
    'notice' AS id,
    'Saved' AS title,
    'green' AS color,
    FALSE AS visible
WHERE $scenario = 'closed' AND $visible IS NOT NULL;

SELECT 'It works !' AS contents_md
WHERE $scenario = 'closed';

SELECT 'text' AS component
WHERE $scenario = 'closed';

SELECT 'Open notice' AS contents, '#notice' AS link
WHERE $scenario = 'closed';

-- White maps to the light foreground. An empty or omitted color adds none.
SELECT 'modal' AS component, 'omitted' AS id, 'Omitted' AS title
WHERE $scenario = 'colors';

SELECT 'Plain' AS contents
WHERE $scenario = 'colors';

SELECT 'modal' AS component, 'empty' AS id, 'Empty' AS title, '' AS color
WHERE $scenario = 'colors';

SELECT 'Plain' AS contents
WHERE $scenario = 'colors';

SELECT 'modal' AS component,
    'white' AS id,
    'White' AS title,
    'white' AS color,
    TRUE AS visible
WHERE $scenario = 'colors';

SELECT 'Plain' AS contents
WHERE $scenario = 'colors';

-- Two modals ask to open on load; only the later one does.
SELECT 'modal' AS component, 'first' AS id, 'First' AS title, TRUE AS visible
WHERE $scenario = 'two';

SELECT 'Earlier' AS contents
WHERE $scenario = 'two';

SELECT 'modal' AS component, 'second' AS id, 'Second' AS title, TRUE AS visible
WHERE $scenario = 'two';

SELECT 'Later' AS contents
WHERE $scenario = 'two';

-- A hash that names a different modal wins over `visible`.
SELECT 'modal' AS component, 'notice' AS id, 'Saved' AS title, TRUE AS visible
WHERE $scenario = 'hash-modal';

SELECT 'Visible' AS contents
WHERE $scenario = 'hash-modal';

SELECT 'modal' AS component, 'other' AS id, 'Other' AS title
WHERE $scenario = 'hash-modal';

SELECT 'Hashed' AS contents
WHERE $scenario = 'hash-modal';

-- A hash that names a non-modal anchor still allows `visible` to open.
SELECT 'text' AS component, 'section' AS id
WHERE $scenario = 'hash-anchor';

SELECT 'In the page' AS contents
WHERE $scenario = 'hash-anchor';

SELECT 'modal' AS component, 'notice' AS id, 'Saved' AS title, TRUE AS visible
WHERE $scenario = 'hash-anchor';

SELECT 'It works !' AS contents_md
WHERE $scenario = 'hash-anchor';
