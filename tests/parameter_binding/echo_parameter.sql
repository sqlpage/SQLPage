select 'text' as component, max(n) as contents from (select $x as n) as t;
