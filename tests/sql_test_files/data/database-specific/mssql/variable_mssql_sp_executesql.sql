-- https://github.com/sqlpage/SQLPage/issues/516
-- sp_executesql is used instead of CONTAINS: both reject a CAST expression as an
-- argument, but CONTAINS needs a full-text index, which cannot be created on temp tables.
SET x = 'It works !';
exec sp_executesql N'SELECT @p as actual, @exp as expected', N'@p varchar(100), @exp varchar(100)', @p=$x, @exp='It works !';
