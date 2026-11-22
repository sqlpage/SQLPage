select 'status_code' as component, 404 as status
where $function is not null and not exists (select 1 from sqlpage_functions where name = $function);

select 'http_header' as component,
    printf('<%s>; rel="canonical"',
        iif($function is not null, sqlpage.link('functions', json_object('function', $function)), 'functions.sql')
    ) as "Link";

set outline = case when $function is null then
    '[{"title":"Overview","link":"#overview"},{"title":"Function catalog","link":"#catalog"},{"title":"Evaluation","link":"#evaluation"}]'
    when exists (select 1 from sqlpage_function_parameters where "function" = $function) then
    '[{"title":"Overview","link":"#function"},{"title":"Parameters","link":"#parameters"}]'
    when exists (select 1 from sqlpage_functions where name = $function) then
    '[{"title":"Overview","link":"#function"}]'
end;

select 'dynamic' as component, sqlpage.run_sql('sqlpage/docs-shell.sql', json_object(
    'title', coalesce('sqlpage.' || $function, 'Functions') || ' - SQLPage Documentation',
    'section', 'Functions', 'current', $function, 'outline', $outline
)) as properties;

select 'empty_state' as component, 'Function not found' as title, 'function' as icon,
    'Browse functions' as link_text, '/functions.sql' as link
where $function is not null and not exists (select 1 from sqlpage_functions where name = $function);

select 'docs-reference-heading' as component, 'overview' as id,
    'Documentation' as section, '/documentation.sql' as link, 'function' as icon,
    'SQLPage functions' as title,
    'Use SQLPage functions alongside the functions provided by your database to read requests, work with files, and call other services.' as description
where $function is null;

select 'title' as component, 'Function catalog' as contents, 2 as level, 'catalog' as id where $function is null;

select 'docs-catalog' as component where $function is null;
with entries as (
    select name, icon, ltrim(trim(description_md, char(10) || ' '), '# ') as description
    from sqlpage_functions where $function is null
)
select name, icon, substr(description, 1, instr(description || char(10), char(10)) - 1) as summary_md,
    sqlpage.link('/functions.sql', json_object('function', name)) as link
from entries order by name;

select 'title' as component, 'How SQLPage evaluates functions' as contents, 2 as level, 'evaluation' as id where $function is null;
select 'text' as component, true as article, '
SQLPage evaluates its built-in functions itself, either before or after the database query.

When a SQLPage function call is the whole value of a selected column, SQLPage first lets the database decide which rows exist.
It selects the function arguments as hidden columns, then calls the SQLPage function once for each returned row.
For example, `SELECT sqlpage.url_encode(url) AS encoded FROM t` runs the database query first, then applies `url_encode` to every returned `url` value.
If the query returns no row, the function is not called.
This also applies inside scalar `SET` subqueries, for instance `SET body = (SELECT sqlpage.fetch(url) FROM cache_misses WHERE enabled)`.

In other positions, SQLPage functions run before the query, but only when their arguments can be evaluated without reading database columns.
For instance, `WHERE sqlpage.cookie(''x'') = ''1''` can run before the query, but `WHERE sqlpage.fetch(url) IS NOT NULL` cannot because `url` is a database column.

If a SQLPage function is expensive or has side effects and should run only once, store its result with `SET` first and reuse the variable.
If a function should run only when a database row exists, put it as a standalone selected column in that row-producing query.

For more information about how SQLPage functions are evaluated, and data types in SQLPage, read [the SQLPage data model documentation](/extensions-to-sql).
' as contents_md where $function IS NULL;

select 'docs-reference-heading' as component, 'function' as id,
    'Function reference' as section, '/functions.sql' as link, f.icon,
    'sqlpage.' || f.name || '(' || coalesce((
        select group_concat(p.name, ', ' order by p."index")
        from sqlpage_function_parameters as p where p."function" = f.name
    ), '') || ')' as title, f.introduced_in_version as version, f.return_type
from sqlpage_functions as f where f.name = $function;

select 'text' as component, true as article, description_md as contents_md
from sqlpage_functions where name = $function;

select 'docs-parameters' as component, 'Parameters' as title, 'parameters' as id
where exists (select 1 from sqlpage_function_parameters where "function" = $function);
select name, type, description_md
from sqlpage_function_parameters where "function" = $function order by "index";
