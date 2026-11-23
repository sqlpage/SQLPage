select 'shell-docs' as component, 'Configuration | SQLPage' as title, 'Configuration' as section;

select 'docs-reference-heading' as component, 'configuration' as id, 'Documentation' as section,
    '/documentation.sql' as link, 'settings' as icon, 'Configuration' as title,
    'Server settings, accepted values, and defaults.' as description;

select 'text' as component,
    'Set options in your configuration file or use their uppercase names as environment variables. The [configuration guide](https://github.com/sqlpage/SQLPage/blob/main/configuration.md) covers file formats, connection strings, and setup examples.' as contents_md;

select 'docs-parameters' as component;
select 'options' as id, 'Options' as title, json_group_array(json_object(
    'name', name, 'anchor', name, 'type', type, 'description_md', description_md,
    'default_md', coalesce(default_md, 'Unset')
) order by name) as parameters from config_option;
