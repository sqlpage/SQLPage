SELECT 'shell' as component, 'SQLPage' as title,
    'chart' as menu_item,
    'layouts' as menu_item,
    'tabs' as menu_item,
    'show_variables' as menu_item;

select 'list' as component, 'POST variables' as title,
 'Here is the list of POST variables sent to this page.
 Post variables are accessible with `:variable_name`.' as description_md,
 'No POST variable.' as empty_title;
select key as title, ':' || key || ' = ' || "value" as description
from json_each(sqlpage.variables('post'));

select 'list' as component, 'GET variables' as title,
 'Here is the list of GET variables sent to this page.
 Get variables are accessible with `$variable_name`.' as description_md,
 'No GET variable.' as empty_title;
select key as title, '$' || key || ' = ' || "value" as description
from json_each(sqlpage.variables('get'));