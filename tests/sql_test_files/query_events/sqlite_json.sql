select json('[1]') as values_json, json('[2]') as values_json,
    sqlpage.url_encode(name) as encoded
from (select 'a b' as name) source_rows;
