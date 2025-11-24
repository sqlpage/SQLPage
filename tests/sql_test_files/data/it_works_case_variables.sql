-- https://github.com/sqlpage/SQLPage/issues/818

set success = 'It works !';
set failure = 'You should never see this';

select 'It works !' as expected,
    case $success
        when $success then $success
        when $failure then $failure
    end as actual;