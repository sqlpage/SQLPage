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
    ('item', 'A list of bullet points associated with the columns, represented either as text, or as a json object with "icon", "color", and "description" or "description_md" fields.', 'JSON', FALSE, TRUE),
    ('link', 'A link associated with the item.', 'TEXT', FALSE, TRUE),
    ('button_text', 'Text for the button.', 'TEXT', FALSE, TRUE),
    ('button_color', 'Optional color for the button.', 'TEXT', FALSE, TRUE),
    ('target', 'Optional target for the button. Set to "_blank" to open links in a new tab.', 'TEXT', FALSE, TRUE),
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
            "title":"Start Plan",
            "value":"€18",
            "description":"Perfect for testing and small-scale projects",
            "item": [
                "128MB Database",
                "SQLPage hosting",
                "Community support"
            ],
            "link":"https://datapage.app",
            "button_text":"Start Free Trial",
            "small_text":"/month"
        },
        {
            "title":"Pro Plan",
            "value":"€40",
            "icon":"rocket",
            "description":"For growing projects needing enhanced features",
            "item": [
                {"icon":"database", "color": "blue", "description":"1GB Database"},
                {"icon":"headset", "color": "green", "description":"Priority Support"},
                {"icon":"world", "color": "purple", "description":"Custom Domain"}
            ],
            "link":"https://datapage.app",
            "button_text":"Start Free Trial",
            "button_color":"indigo",
            "value_color":"indigo",
            "small_text":"/month"
        },
        {
            "title":"Enterprise Plan",
            "value":"€600",
            "icon":"building-skyscraper",
            "description":"For large-scale operations with custom needs",
            "item": [
                {"icon":"database-plus", "description_md":"**Custom Database Scaling**"},
                {"icon":"shield-lock", "description_md":"**Enterprise Auth** with Single Sign-On"},
                {"icon":"headset", "description_md":"**Monthly** Expert Support time"},
                {"icon":"file-certificate", "description_md":"**SLA** with guaranteed uptime"}
            ],
            "link":"mailto:contact@datapage.app",
            "button_text":"Contact Us",
            "small_text":"/month",
            "target":"_blank"
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
