-- in SQLite, we provide our own unicode-aware lower function
-- see https://github.com/sqlpage/SQLPage/issues/452
select 'It works !' as expected,
    coalesce(lower(NULL), 'It works !') as actual;