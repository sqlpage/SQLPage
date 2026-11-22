-- Render the common parts of a component or function reference from its catalog entry.
set options = sqlpage.variables('set');
set kind = $options->>'kind';
set name = $options->>'name';
set section = case $kind when 'component' then 'Components' else 'Functions' end;

set outline = (
    select json_group_array(json_object('title', title, 'link', '#' || id) order by position)
    from (
        select 0 as position, 'Overview' as title, kind as id from reference_entry where kind = $kind and name = $name
        union all
        select distinct section_order, title, section from reference_parameter where kind = $kind and entry = $name
        union all
        select 3, 'Examples', 'examples' from reference_entry where kind = $kind and name = $name and example_count > 0
    )
);

select 'dynamic' as component, sqlpage.run_sql('sqlpage/docs-shell.sql', json_object(
    'title', case $kind when 'function' then 'sqlpage.' || $name else $name end || ' - SQLPage Documentation',
    'section', $section, 'current', $name, 'outline', $outline
)) as properties;

select 'docs-reference-heading' as component, kind as id,
    case kind when 'component' then 'Component reference' else 'Function reference' end as section,
    '/' || lower($section) || '.sql' as link,
    case kind when 'component' then 'The ' || name || ' component'
        else 'sqlpage.' || name || '(' || arguments || ')' end as title,
    icon, introduced_in_version as version, return_type,
    case when example_count > 0 then '#examples' end as jump_link, 'View examples' as jump_text
from reference_entry where kind = $kind and name = $name;

select 'text' as component, true as article, description_md as contents_md
from reference_entry where kind = $kind and name = $name;

select 'docs-parameters' as component;
select section as id, title, introduction, json_group_array(json_object(
    'name', name, 'type', type, 'required', not optional, 'description_md', description_md, 'anchor', anchor
) order by position, name) as parameters
from reference_parameter where kind = $kind and entry = $name
group by section_order, section, title, introduction order by section_order;
