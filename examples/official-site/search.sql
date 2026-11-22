set search = nullif(trim($search), '');

-- An unqualified name shared by a component and a function opens the component.
select 'redirect' as component, link from reference_entry
where name = lower($search) or (kind = 'function' and lower($search) in (
    name || '()', 'sqlpage.' || name, 'sqlpage.' || name || '()'
))
order by kind limit 1;

select 'shell-docs' as component, coalesce($search || ' | ', '') || 'SQLPage documentation search' as title;

select 'title' as component, 'Search documentation' as contents, 1 as level;

select 'form' as component, 'GET' as method, 'Search' as validate;
select 'search' as type, 'search' as name, 'Search the documentation' as label,
    'Tutorials, components, functions…' as placeholder, true as autofocus, $search as value;

-- Treat the input as a literal phrase in SQLite's full-text query syntax.
set escaped_search = '"' || replace(coalesce($search, ''), '"', '""') || '"';

select 'list' as component, 'Search results' as title, 'No results' as empty_title,
    'No documentation matched “' || $search || '”.' as empty_description
where $search is not null;

with matches as (
    select title, trim(description, char(13) || char(10) || ' ') as summary, page, parameters, fragment, rank,
        row_number() over (partition by page, parameters order by rank, rowid) as position
    from documentation_fts
    where documentation_fts match $escaped_search
)
select title,
    ltrim(substr(summary, 1, instr(summary || char(10), char(10)) - 1), '# ') as description,
    sqlpage.link(page, parameters, fragment) as link
from matches
where position = 1
order by rank, title;
