SELECT
    '[{"label" : "P123", "value" : "Jaws"}]' AS expected,
    json_agg(json_object('label' VALUE 'P123', 'value' : 'Jaws'))::text AS actual;
