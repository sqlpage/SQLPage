DROP TABLE IF EXISTS component;
CREATE TABLE component(
    name TEXT PRIMARY KEY,
    description TEXT
);

DROP TABLE IF EXISTS parameter;
CREATE TABLE parameter(
    name TEXT PRIMARY KEY,
    component TEXT REFERENCES component(name),
    description TEXT,
    type TEXT,
    optional BOOL DEFAULT FALSE
);

DROP TABLE IF EXISTS example_value;
CREATE TABLE example_value(
    component TEXT REFERENCES component(name),
    parameter TEXT REFERENCES parameter(name),
    value TEXT
);

select
    'SQLPage documentation' as title,
    '/' as link,
    'en' as lang,
    'SQLPage documentation' as description;


select 'text' as component, 'SQLPage documentation' as title;
select 'Building an application with SQLPage is quite simple.' ||
    'To create a new web page, just create a new SQL file. ' ||
    'For each SELECT statement that you write, the data it returns will be analyzed and rendered to the user.';
select 'The two most important concepts in SQLPage are ' as contents;
select 'components' as contents, true as bold;
select ' and ' as contents;
select 'parameters' as contents, true as bold;
select '.' as contents;
select 'This page documents all the components that you can use in SQLPage and their parameters. ' ||
     'Use this as a reference when building your SQL application.' as contents;
