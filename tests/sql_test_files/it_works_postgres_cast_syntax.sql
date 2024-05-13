-- the syntax $x::int is supported only in PostgreSQL
-- but for consistency with other databases, sqlpage supports this syntax everywher ()
SELECT 'text' as component, 
    case $x::integer + 1 
        when 2 then 'It works !'
        else 'Error !'
    end as contents;
