set options = sqlpage.variables('set');
set section = $options->>'section';
set kind = case $section when 'Components' then 'component' when 'Functions' then 'function' end;
set catalog = $kind is not null and $options->>'current' is null;

select 'shell-docs' as component,
    $options->>'title' as title,
    $options->>'description' as description,
    $section as section,
    cast($catalog as integer) as catalog,
    coalesce($kind || 's', 'guides') as topic_label,
    '/' || $kind || 's.sql' as catalog_link,
    $options->>'current' as current_item,
    nullif($options->>'outline', '[]') as outline,
    case
        when $catalog then null
        when $kind is not null then (
            select json_group_array(json_object('title', name, 'link', link) order by name)
            from reference_entry where kind = $kind
        )
        else (
            select json_group_array(json_object('title', title, 'link', link) order by sort_order) from guide
        )
    end as topics;
