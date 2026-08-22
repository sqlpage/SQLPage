-- https://github.com/sqlpage/SQLPage/issues/1154
select 'It works !' as expected, case when $x = 1 then 'It works !' else 'fail' end as actual;
