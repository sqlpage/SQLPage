set res = sqlpage.fetch('{
  "url": "http://localhost:' || $echo_port || '/hello_world",
  "response_encoding": "hex"
}');
select 'text' as component,
    case
        when $res LIKE '474554202f68656c6c6f5f776f726c64%' then 'It works !'
        else 'It failed ! Got: ' || $res
    end as contents;
