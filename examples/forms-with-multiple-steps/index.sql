select 'list' as component, 'Forms with multiple steps' as title;

select 'Database persistence' as title, 'database' as link;
select 'Cookies' as title, 'cookies' as link;
select 'Hidden fields' as title, 'hidden' as link;

select 'text' as component, sqlpage.read_file_as_text('README.md') as contents_md;