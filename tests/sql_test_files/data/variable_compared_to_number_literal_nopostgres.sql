-- Comparing a variable to a numeric literal must work on every database.
-- This exercises the text cast that SQLPage keeps around variables on
-- databases that cannot infer a text parameter type from the query context.
select 'It works !' as expected,
    case when $x = 1 then 'It works !' else 'It does not work' end as actual;
