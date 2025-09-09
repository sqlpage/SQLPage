SET msg = case when 1=1 then 'It works !' else 'It failed !' end;
select 'text' as component, $msg AS contents;
