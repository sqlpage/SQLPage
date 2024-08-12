set my_decimal = CAST(0.47 AS DECIMAL(3,2));

select 'text' as component, 
    case $my_decimal
        when '0.47' then 'It works !'
        else 'error: ' || coalesce($my_decimal, 'NULL')
    end AS contents;