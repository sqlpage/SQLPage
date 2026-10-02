SELECT 'dynamic' AS component, properties FROM example WHERE component = 'shell' LIMIT 1;
SELECT 'text' AS component, sqlpage.read_file_as_text('your-first-sql-website/service.md') AS contents_md;
