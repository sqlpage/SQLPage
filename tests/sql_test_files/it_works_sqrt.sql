set number_three = sqrt(9.0);

select 'text' as component, 
    case $number_three
        when '3.0' then 'It works !'
        when '3' then 'It works !'
        else 'error: ' || coalesce($number_three, 'NULL')
    end AS contents;
