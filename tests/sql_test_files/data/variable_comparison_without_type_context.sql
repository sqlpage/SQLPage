-- Context-free variables (no column or literal) need CAST on SQLite and psqlodbc
SET other = 'other';
select 'It works !' as expected, 'It works !' as actual where $x <> $other or $x is null;
