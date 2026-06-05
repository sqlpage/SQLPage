select
    N'csv' as component,
    N';' as separator;

select
    0 as id,
    N'Hello World !' as msg
union all
select
    1 as id,
    CONCAT(N'Tu gères ', NCHAR(39), NCHAR(59), NCHAR(39), N' et ', NCHAR(39), NCHAR(34), NCHAR(39), N' ?') as msg;
