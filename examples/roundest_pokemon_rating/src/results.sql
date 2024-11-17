select 'results' as component;

with ranked as (
    select
        *,
        case 
            when (up_votes + down_votes) = 0 then 0
            else 100 * up_votes / (up_votes + down_votes)
        end as win_percentage,
        up_votes - down_votes as score
    from pokemon
)
select *, rank() over (order by score desc) as rank
from ranked
order by win_percentage desc, score desc;
