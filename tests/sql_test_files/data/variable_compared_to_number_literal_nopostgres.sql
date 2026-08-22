-- Variable compared to numeric literal must work (SQLite/ODBC need CAST)
select 'It works !' as expected, case when $x = 1 then 'It works !' else 'fail' end as actual;
