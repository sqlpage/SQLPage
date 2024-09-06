-- Column Component Documentation

-- Component Definition
INSERT INTO component(name, icon, description, introduced_in_version) VALUES
    ('columns', 'columns', 'A component to display various items in a card layout, allowing users to choose options. Useful for showcasing different features or services, or KPIs. See also the big_number component.', '0.29.0');

-- Inserting parameter information for the column component
INSERT INTO parameter(component, name, description, type, top_level, optional) SELECT 'columns', * FROM (VALUES
    ('title', 'The title or label for the item.', 'TEXT', FALSE, TRUE),
    ('value', 'The value associated with the item.', 'TEXT', FALSE, TRUE),
    ('description', 'A brief description of the item.', 'TEXT', FALSE, TRUE),
    ('description_md', 'A brief description of the item, formatted using markdown.', 'TEXT', FALSE, TRUE),
    ('item', 'A list of bullet points associated with the columns, represented as JSON.', 'JSON', FALSE, TRUE),
    ('link', 'A link associated with the item.', 'TEXT', FALSE, TRUE),
    ('button_text', 'Text for the button.', 'TEXT', FALSE, TRUE),
    ('button_color', 'Optional color for the button.', 'TEXT', FALSE, TRUE),
    ('value_color', 'Color for the value text.', 'TEXT', FALSE, TRUE),
    ('small_text', 'Optional small text to display after the value.', 'TEXT', FALSE, TRUE),
    ('icon', 'Optional icon to display in a ribbon.', 'ICON', FALSE, TRUE),
    ('icon_color', 'Color for the icon in the ribbon.', 'TEXT', FALSE, TRUE),
    ('size', 'Size of the column, affecting layout.', 'INTEGER', FALSE, TRUE)
) x;

INSERT INTO example(component, description, properties) VALUES
    ('columns', 'Pricing Plans Display',
    json('[
        {"component":"columns"},
        {
            "title":"Basic Plan",
            "value":"$0",
            "description":"A basic plan for individuals.",
            "item": [
                {"description":"Creation & Edition"},
                {"description":"Limited Customization"}
            ],
            "link":"#",
            "button_text":"Select",
            "small_text":"/month"
        },
        {
            "title":"Standard Plan",
            "value":"$49",
            "icon":"star",
            "description":"A standard plan for small teams.",
            "item": [
                {"icon":"check", "color": "green", "description":"Collaboration Tools"},
                {"icon":"check", "color": "green", "description":"Custom data sources"},
                {"icon":"x", "color": "red", "description":"Priority support"}
            ],
            "link":"#",
            "button_text":"Select",
            "button_color":"success",
            "value_color":"green",
            "small_text":"/month"
        },
        {
            "title":"Premium Plan",
            "value":"$99",
            "description":"A premium plan for larger teams.",
            "item": [
                {"icon":"check", "color": "green", "description":"Collaboration Tools"},
                {"icon":"check", "color": "green", "description":"Custom data sources"},
                {"icon":"check", "color": "green", "description":"Priority support"}
            ],
            "link":"#",
            "button_text":"Select",
            "small_text":"/month"
        }
    ]')),
    
    ('columns', 'Tech Company KPIs Display',
    json('[
        {"component":"columns"},
        {
            "title":"Monthly Active Users",
            "value":"10k",
            "value_color":"blue",
            "size": 4,
            "description":"Total active users this month, showcasing user engagement.",
            "item": [
                {"icon": "target", "description":"Target: 12,000"}
            ],
            "link":"#",
            "button_text":"User Activity Overview",
            "button_color":"info"
        },
        {
            "title":"Revenue",
            "value":"$49k",
            "value_color":"blue",
            "size": 4,
            "description":"Total revenue generated this month, indicating financial performance.",
            "item": [
                {"icon":"trending-down", "color": "red", "description":"down from $51k last month" }
            ],
            "link":"#",
            "button_text":"Financial Dashboard",
            "button_color":"info"
        },
        {
            "title":"Customer Satisfaction",
            "value":"94%",
            "value_color":"blue",
            "size": 4,
            "description":"Percentage of satisfied customers, reflecting service quality.",
            "item": [
                {"icon":"trending-up", "color": "green", "description":"+ 2% this month" }
            ],
            "link":"#",
            "button_text": "Open Google Ratings",
            "button_color":"info"
        }
    ]'));
