-- Documentation for the probe component
INSERT INTO component (name, description, icon, introduced_in_version) VALUES (
    'probe',
    'A debug component that displays a value (constant, variable, parameter value, result of an SQL function, e.g.). Usefull to validate data flow or function behaviour. Each value (except NULL value) is surrounded by double quotes and followed by datatype (number, string, boolean, e.g.).',
    'microscope',
    '0.20.0'
);

INSERT INTO parameter (component,name,description,type,top_level,optional) VALUES (
    'probe',
    'name',
    'Value identifier (used to display the probe result).',
    'TEXT',
    TRUE,
    FALSE
),(
    'probe',
    'contents',
    'A value to analyze by the probe.',
    'VARIANT',
    TRUE,
    FALSE
);

-- Insert example(s) for the component
INSERT INTO example(component, description, properties) VALUES (
    'probe',
    'Analyse a number.',
    JSON(
        '[
            {"component":"probe","name":"myValue","contents":42}
        ]'
    )
);

INSERT INTO example(component, description, properties) VALUES (
    'probe',
    'Analyse a string.',
    JSON(
        '[
            {"component":"probe","name":"myString","contents": "Hello world! "}
        ]'
    )
);

INSERT INTO example(component, description, properties) VALUES (
    'probe',
    'Analyse a NULL value.',
    JSON(
        '[
            {"component":"probe","name":"myVar","contents": null}
        ]'
    )
);