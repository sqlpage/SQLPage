-- Big Number Component Documentation

-- Component Definition
INSERT INTO component(name, icon, description, introduced_in_version) VALUES
    ('big_number', 'chart-area', 'A component to display key metrics or statistics with optional description, change indicator, and progress bar. Useful in dashboards.', '0.28.0');

-- Inserting parameter information for the big_number component
INSERT INTO parameter(component, name, description, type, top_level, optional) SELECT 'big_number', * FROM (VALUES
    -- Top-level parameters (for the whole big_number list)
    ('columns', 'The number of columns to display the big numbers in (default is one column per item).', 'INTEGER', TRUE, TRUE),
    ('id', 'An optional ID to be used as an anchor for links.', 'TEXT', TRUE, TRUE),
    ('class', 'An optional CSS class to be added to the component for custom styling', 'TEXT', TRUE, TRUE),
    -- Item-level parameters (for each big number)
    ('title', 'The title or label for the big number.', 'TEXT', FALSE, TRUE),
    ('title_link', 'A link for the Big Number title. If set, the entire title becomes clickable.', 'TEXT', FALSE, TRUE),
    ('title_link_new_tab', 'If true, the title link will open in a new tab/window.', 'BOOLEAN', FALSE, TRUE),
    ('value_link', 'A link for the Big Number value. If set, the entire value becomes clickable.', 'TEXT', FALSE, TRUE),
    ('value_link_new_tab', 'If true, the value link will open in a new tab/window.', 'BOOLEAN', FALSE, TRUE),
    ('value', 'The main value to be displayed prominently.', 'TEXT', FALSE, FALSE),
    ('unit', 'The unit of measurement for the value.', 'TEXT', FALSE, TRUE),
    ('description', 'A description or additional context for the big number.', 'TEXT', FALSE, TRUE),
    ('change_percent', 'The percentage change in value (e.g., 7 for 7% increase, -8 for 8% decrease).', 'INTEGER', FALSE, TRUE),
    ('progress_percent', 'The value of the progress (0-100).', 'INTEGER', FALSE, TRUE),
    ('progress_color', 'The color of the progress bar (e.g., "primary", "success", "danger").', 'TEXT', FALSE, TRUE),
    ('dropdown_item', 'A list of JSON objects containing links. e.g. {"label":"This week", "link":"?days=7"}', 'JSON', FALSE, TRUE),
    ('color', 'The color of the card', 'COLOR', FALSE, TRUE)
) x;

INSERT INTO example(component, description, properties) VALUES
    ('big_number', 'Big numbers with change indicators and progress bars',
    json('[
        {"component":"big_number"},
        {
            "title":"Sales",
            "value":75,
            "unit":"%",
            "title_link": "#sales_dashboard",
            "title_link_new_tab": true,
            "value_link": "#sales_details",
            "value_link_new_tab": false,
            "description":"Conversion rate",
            "change_percent": 9,
            "progress_percent": 75,
            "progress_color": "blue"
        },
        {
            "title":"Revenue",
            "value":"4,300",
            "unit":"$",
            "description":"Year on year",
            "change_percent": -8
        }
    ]'));

INSERT INTO example(component, description, properties) VALUES
    ('big_number', 'Big numbers with dropdowns and customized layout',
    json('[
        {"component":"big_number", "columns":3, "id":"colorfull_dashboard"},
        {"title":"Users", "value":"1,234", "color": "red", "title_link": "#users", "title_link_new_tab": false, "value_link": "#users_details", "value_link_new_tab": true },
        {"title":"Orders", "value":56, "color": "green", "title_link": "#orders", "title_link_new_tab": true },
        {"title":"Revenue", "value":"9,876", "unit": "€", "color": "blue", "change_percent": -7, "dropdown_item": [
            {"label":"This week", "link":"?days=7&component=big_number#colorfull_dashboard"},
            {"label":"This month", "link":"?days=30&component=big_number#colorfull_dashboard"},
            {"label":"This quarter", "link":"?days=90&component=big_number#colorfull_dashboard"}
        ]}
    ]'));
