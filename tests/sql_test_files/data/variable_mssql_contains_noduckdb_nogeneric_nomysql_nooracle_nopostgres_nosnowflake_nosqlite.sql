-- https://github.com/sqlpage/SQLPage/issues/516
SET x = 'It works !';
exec sp_executesql N'SELECT ''It works !'' as expected, @p as actual', N'@p varchar(100)', @p=$x;
