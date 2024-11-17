update pokemon
set
    up_votes = up_votes + (dex_id = $voted),
    down_votes = down_votes + (dex_id != $voted)
where dex_id IN (:option_0, :option_1);

select 'redirect' as component, '/' as link;
