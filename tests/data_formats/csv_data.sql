select
    'csv' as component,
    ';' as separator;

select
    0 as "ID",
    'Hello World !' as "MSG"
union all
select
    1 as "ID",
    'Tu gères '';'' et ''"'' ?' as "MSG";