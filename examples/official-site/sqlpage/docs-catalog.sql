set options = sqlpage.variables('set');
set kind = $options->>'kind';
set query = trim(coalesce($options->>'q', ''));
set search = lower(case when $kind = 'function' and lower($query) like 'sqlpage.%'
    then substr($query, 9) else $query end);
set action = case $kind when 'component' then '/components.sql' else '/functions.sql' end;
set search_url = $action || case when $query = '' then '' else '?q=' || sqlpage.url_encode($query) end;

-- Keep the counts for every category when showing only the selected category's rows.
set groups = (
    with candidates as (
        select *,
            instr(lower(name || coalesce('(' || arguments || ')', '') || char(10) ||
                replace(name, '_', ' ') || char(10) || coalesce(summary, '')), $search) > 0
            or ($kind = 'function' and $search in (
                name || '()', name || '(' || replace(replace(arguments, '[', ''), ']', '') || ')'
            )) as matches
        from reference_entry where kind = $kind
    ), categories as (
        select category_id, category_title, category_order,
            json_group_array(json_object(
                'name', name, 'icon', icon, 'summary', summary, 'link', link,
                'arguments', arguments, 'example_count', example_count
            ) order by name) filter (where matches) as entries
        from candidates
        group by category_id, category_title, category_order
    )
    select json_group_array(json_object(
        'id', category_id, 'title', category_title,
        'count', json_array_length(entries), 'entries', json(entries)
    ) order by category_order)
    from categories
);
set category = coalesce((
    select value->>'id' from json_each($groups) where value->>'id' = $options->>'category'
), '');

select 'docs-catalog' as component, $kind as kind, $kind || 's' as plural,
    $query as query, $category as category, $action as action,
    $action || '#catalog' as clear_link, $search_url || '#catalog' as all_link,
    max(value->>'title') filter (where value->>'id' = $category) as category_title,
    coalesce(sum(value->>'count'), 0) as all_count,
    coalesce(sum(value->>'count') filter (where $category = '' or value->>'id' = $category), 0) as matched,
    json_group_array(json_object(
        'id', value->>'id', 'title', value->>'title', 'count', value->>'count',
        'active', value->>'id' = $category,
        'link', $search_url || case when $query = '' then '?' else '&' end
            || 'category=' || (value->>'id') || '#catalog'
    ) order by key) as categories,
    json(case when $kind = 'component' then (
        select json_group_array(json_object('name', name, 'link', link)
            order by case name when 'table' then 1 when 'form' then 2 else 3 end)
        from reference_entry where kind = 'component' and name in ('table', 'form', 'chart')
    ) end) as picks,
    case when $kind = 'function' then sqlpage.url_encode('hello world') end as encoded_example
from json_each($groups);

select value->>'id' as id, value->>'title' as title, json(value->'entries') as entries
from json_each($groups)
where value->>'count' > 0 and ($category = '' or value->>'id' = $category)
order by key;
