SET x = 42;
SET y = $x;
SET z = $y;
select 'text' as component,
    case $z 
        when '42' then 'It works !'
        else 'It failed ! Expected 42 but got ' || COALESCE($z, 'NULL')
    end
AS contents;