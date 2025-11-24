select 'test.sql?x=123' as expected, sqlpage.link('test.sql', json_object('x', 123)) as actual;
