-- https://github.com/sqlpage/SQLPage/issues/516 (psqlodbc needs CAST for context-free params)
SET other = 'other';
select 'It works !' as expected, 'It works !' as actual where $x <> $other or $x is null;
