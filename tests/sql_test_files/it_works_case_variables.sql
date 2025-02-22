-- https://github.com/sqlpage/SQLPage/issues/818

set success = 'It works !';
set failure = 'You should never see this';

select 'text' as component, 
    case $success
        when $success then $success
        when $failure then $failure
    end AS contents;