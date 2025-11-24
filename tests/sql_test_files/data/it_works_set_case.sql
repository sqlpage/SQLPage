set msg = case when 1=1 then 'It works !' else 'It failed !' end;
select 'It works !' as expected, $msg as actual;
