select 'status_code' as component, 404 as status;
select 'http_header' as component, 'noindex' as "X-Robots-Tag";
select 'shell-docs' as component, 'Component not found - SQLPage' as title;

select 'text' as component, 'Component not found' as title,
    'Browse the [component catalog](/components.sql) to find available components and examples.' as contents_md;

set show_security_note = $component like '%<%' or $component like '%>%' or $component like '%/%'
    or $component like '%;%' or $component like '%--%' or $component like '%''%' or $component like '%(%';

select 'alert' as component, 'A note about security' as title,
    'alert-triangle' as icon, 'teal' as color, true as important,
    'SQLPage treats component names as data. If you think you have found a vulnerability, please report it privately to the maintainers.' as description
where $show_security_note;

select '/safety.sql' as link, 'More about SQLPage security' as title where $show_security_note;
select 'https://github.com/sqlpage/SQLPage/security' as link, 'Report a vulnerability' as title where $show_security_note;
