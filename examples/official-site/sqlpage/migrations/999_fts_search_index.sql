CREATE VIRTUAL TABLE documentation_fts USING fts5(
    component_name,
    component_description,
    parameter_name,
    parameter_description,
    blog_title,
    blog_description,
    function_name,
    function_description,
    function_parameter_name,
    function_parameter_description,
    component_example_description,
    component_example_json
);

INSERT INTO documentation_fts(component_name, component_description)
SELECT name, description FROM component;

INSERT INTO documentation_fts(component_name, parameter_name, parameter_description)
SELECT component, name, description FROM parameter;

INSERT INTO documentation_fts(blog_title, blog_description)
SELECT title, description FROM blog_posts;

INSERT INTO documentation_fts(function_name, function_description)
SELECT name, description_md FROM sqlpage_functions;

INSERT INTO documentation_fts(function_name, function_parameter_name, function_parameter_description)
SELECT function, name, description_md FROM sqlpage_function_parameters;

INSERT INTO documentation_fts(component_name, component_example_description, component_example_json)
SELECT component, description, properties FROM example;