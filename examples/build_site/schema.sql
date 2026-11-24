CREATE TABLE reference_category (
    id TEXT PRIMARY KEY,
    title TEXT NOT NULL,
    sort_order INTEGER NOT NULL
);

CREATE TABLE config_option (
    name TEXT PRIMARY KEY,
    type TEXT NOT NULL,
    default_md TEXT,
    description_md TEXT NOT NULL
);

CREATE TABLE component (
    name TEXT PRIMARY KEY,
    description TEXT NOT NULL,
    icon TEXT,
    introduced_in_version TEXT,
    catalog_category TEXT REFERENCES reference_category(id),
    catalog_summary TEXT
);

CREATE TABLE parameter_type (name TEXT PRIMARY KEY);
INSERT INTO parameter_type VALUES
    ('BOOLEAN'), ('COLOR'), ('HTML'), ('ICON'), ('INTEGER'), ('JSON'), ('REAL'), ('TEXT'), ('TIMESTAMP'), ('URL');

CREATE TABLE parameter (
    component TEXT NOT NULL REFERENCES component(name),
    top_level BOOLEAN NOT NULL,
    name TEXT NOT NULL,
    type TEXT NOT NULL REFERENCES parameter_type(name),
    description_md TEXT NOT NULL,
    optional BOOLEAN NOT NULL,
    PRIMARY KEY (component, top_level, name)
);

CREATE TABLE example (
    component TEXT NOT NULL REFERENCES component(name),
    description TEXT NOT NULL,
    properties JSON
);

CREATE TABLE sqlpage_functions (
    name TEXT PRIMARY KEY,
    icon TEXT,
    description_md TEXT NOT NULL,
    return_type TEXT,
    introduced_in_version TEXT,
    catalog_category TEXT REFERENCES reference_category(id),
    catalog_summary TEXT
);

CREATE TABLE sqlpage_function_parameters (
    "function" TEXT NOT NULL REFERENCES sqlpage_functions(name),
    "index" INTEGER NOT NULL,
    name TEXT NOT NULL,
    description_md TEXT NOT NULL,
    type TEXT NOT NULL,
    optional BOOLEAN NOT NULL,
    PRIMARY KEY ("function", "index"),
    UNIQUE ("function", name)
);

CREATE TABLE blog_posts (
    title TEXT PRIMARY KEY,
    description TEXT NOT NULL,
    icon TEXT NOT NULL,
    external_url TEXT,
    content TEXT,
    created_at TEXT NOT NULL
);

CREATE TABLE guide (
    sort_order INTEGER PRIMARY KEY,
    title TEXT NOT NULL,
    link TEXT NOT NULL UNIQUE,
    description TEXT NOT NULL
);

CREATE TABLE example_cards (
    title TEXT NOT NULL,
    folder TEXT NOT NULL,
    db_engine TEXT,
    description TEXT NOT NULL
);

-- The reference shows the SQL and renders the same stored component rows.
-- Arrays with several items become repeated column aliases, as in handwritten SQLPage files.
CREATE VIEW example_with_source AS
SELECT example.rowid AS example_id,
    row_number() OVER (PARTITION BY component ORDER BY example.rowid) AS example_number,
    example.*,
    (SELECT group_concat('select' || char(10) || (
        WITH fields AS (
            SELECT field.key AS name, coalesce(item.type, field.type) AS type,
                CASE WHEN item.key IS NULL THEN field.value ELSE item.value END AS value
            FROM json_each(statement.value) AS field
            LEFT JOIN json_each(
                CASE WHEN field.type = 'array' AND json_array_length(field.value) > 1 THEN field.value END
            ) AS item ON true
        )
        SELECT group_concat('    ' || CASE type
                WHEN 'true' THEN 'TRUE'
                WHEN 'false' THEN 'FALSE'
                WHEN 'array' THEN 'JSON(' || quote(value) || ')'
                WHEN 'object' THEN 'JSON(' || quote(value) || ')'
                ELSE quote(value)
            END || format(' as "%w"', name), ',' || char(10))
        FROM fields
    ) || ';', char(10) || char(10)) FROM json_each(example.properties) AS statement) AS source
FROM example;

-- The catalog, topic navigation, and reference pages use the same entries and links.
CREATE VIEW reference_entry AS
WITH entries AS (
    SELECT 'component' AS kind, name, icon, catalog_category, catalog_summary,
        description AS description_md, introduced_in_version, NULL AS return_type,
        '/component.sql?component=' || name AS link,
        (SELECT count(*) FROM example WHERE example.component = component.name) AS example_count,
        NULL AS arguments
    FROM component
    UNION ALL
    SELECT 'function', name, icon, catalog_category, catalog_summary,
        description_md, introduced_in_version, return_type, '/functions.sql?function=' || name, NULL,
        coalesce((SELECT group_concat(CASE WHEN optional THEN '[' || name || ']' ELSE name END, ', ' ORDER BY "index")
            FROM sqlpage_function_parameters WHERE "function" = f.name), '')
    FROM sqlpage_functions AS f
)
SELECT kind, name, icon, link, description_md, introduced_in_version, return_type, example_count, arguments,
    coalesce(category.id, 'other') AS category_id,
    coalesce(category.title, 'Other') AS category_title,
    coalesce(category.sort_order, 99) AS category_order,
    coalesce(catalog_summary, substr(trim(description_md, char(10) || ' #'), 1,
        instr(trim(description_md, char(10) || ' #') || char(10), char(10)) - 1)) AS summary
FROM entries
LEFT JOIN reference_category AS category ON category.id = entries.catalog_category;

-- The section is part of the anchor: a component can accept the same name at both levels.
CREATE VIEW reference_parameter AS
WITH fields AS (
    SELECT 'component' AS kind, component AS entry,
        CASE WHEN top_level THEN 'top-level-parameters' ELSE 'row-level-parameters' END AS section,
        name, type, optional, description_md, optional AS position
    FROM parameter
    UNION ALL
    SELECT 'function', "function", 'parameters', name, type, optional, description_md, "index"
    FROM sqlpage_function_parameters
), sections(section, title, introduction, section_order) AS (VALUES
    ('top-level-parameters', 'Top-level parameters', 'Set these in the SELECT that names the component.', 1),
    ('row-level-parameters', 'Row-level parameters', 'Return these columns in the rows that follow the component.', 2),
    ('parameters', 'Parameters', NULL, 1)
)
SELECT fields.*, title, introduction, section_order, section || '-' || name AS anchor
FROM fields JOIN sections USING (section);
