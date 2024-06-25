-- in SQLite, we provide our own unicode-aware lower function
-- see https://github.com/lovasoa/SQLpage/issues/452
select 'text' as component, COALESCE(lower(NULL), 'It works !') AS contents;