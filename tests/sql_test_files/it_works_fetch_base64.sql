set res = sqlpage.fetch('{
  "url": "http://localhost:' || $echo_port || '/hello_world",
  "response_encoding": "base64"
}');
select 'text' as component,
    case
        when $res LIKE 'R0VUIC9oZWxsb193b3Js%' then 'It works !'
        else 'It failed ! Got: ' || $res
    end as contents;
