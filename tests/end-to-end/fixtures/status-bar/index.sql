SELECT 'status_bar' AS component, 'repository' AS id, 'project-status' AS class,
    'Repository status' AS label, FALSE AS fixed;
SELECT 'Build passed' AS title, 'circle-check' AS icon, 'green' AS color,
    'All checks passed' AS tooltip, '#activity' AS link, 'build-result' AS class;
SELECT 'Branch: feature/improve-search-index' AS title;
SELECT '12 pending changes' AS title;
SELECT 'View history' AS title, '#history' AS link;

SELECT 'list' AS component, 'Recent activity' AS title, 'activity' AS id,
    'min-vh-100' AS class;
SELECT 'Search index updated' AS title;

SELECT 'title' AS component, 'Build history' AS contents, 'history' AS id;

SELECT 'status_bar' AS component;
SELECT 'Connected' AS title;
SELECT 'history' AS icon, 'Open build history' AS tooltip, '#history' AS link;
