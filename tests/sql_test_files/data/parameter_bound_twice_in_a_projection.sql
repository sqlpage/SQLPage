-- One variable bound twice in a single statement, in the projection rather than
-- only in the predicate. https://github.com/sqlpage/SQLPage/issues/1474
drop table if exists parameter_bound_twice_t;
create table parameter_bound_twice_t(id int primary key);
insert into parameter_bound_twice_t (id) values (1);

select 'It works !' as expected,
    case when $x is not null and $x = '1' then 'It works !' else 'fail' end as actual
from parameter_bound_twice_t;

-- Reusing a parameter is scoped to one statement, including SET to NULL.
set x = NULL;
select 'It works !' as expected,
    case when $x is null and $x is null then 'It works !' else 'fail' end as actual
from parameter_bound_twice_t;

set x = '2';
select 'It works !' as expected,
    case when $x = '2' and $x = '2' then 'It works !' else 'fail' end as actual
from parameter_bound_twice_t;
