-- Comparing variables without any surrounding type context (no column, no
-- literal) must work everywhere. This guards the text cast SQLPage keeps
-- around variables on SQLite and ODBC connections, where the database or
-- driver cannot determine the parameter type by itself.
SET other = 'other';

select 'It works !' as expected, 'It works !' as actual
where $x <> $other or $x is null;
