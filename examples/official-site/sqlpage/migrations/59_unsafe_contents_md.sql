INSERT INTO parameter(component, name, description, type, top_level, optional) SELECT 'text', * FROM (VALUES
('unsafe_contents_md','Markdown format with html blocks. Use this only with trusted content. See the html-blocks section of the Commonmark spec for additional info.', 'TEXT', TRUE, TRUE),
('unsafe_contents_md','Markdown format with html blocks. Use this only with trusted content. See the html-blocks section of the Commonmark spec for additional info.', 'TEXT', FALSE, TRUE)
);
