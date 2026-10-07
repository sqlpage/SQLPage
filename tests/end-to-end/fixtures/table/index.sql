SELECT 'text' AS component, 'Component lifecycle' AS contents WHERE $fragment IS NULL;

-- Only the filtering and numeric sorting examples needed from 01_documentation.sql.
SELECT 'dynamic' AS component, JSON('[
    {"component":"table","markdown":"Name","icon":"icon","search":true},
    {"icon":"table","name":"[Table](?component=table)","description":"Displays SQL results as a searchable table.","_sqlpage_color":"red"},
    {"icon":"timeline","name":"[Chart](?component=chart)","description":"Show graphs based on numeric data."},
    {"component":"table","sort":true,"align_right":["Price","Amount in stock"],"align_center":["part_no"],"raw_numbers":["id"],"currency":"USD","money":["Price"]},
    {"id":31456,"part_no":"SQL-TABLE-856-G","Price":12,"Amount in stock":5},
    {"id":996,"part_no":"SQL-FORMS-86-M","Price":1,"Amount in stock":1234},
    {"id":131456,"part_no":"SQL-CARDS-56-K","Price":127,"Amount in stock":98}
]') AS properties
WHERE $fragment IS NOT NULL;
