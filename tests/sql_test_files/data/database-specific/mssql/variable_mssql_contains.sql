-- https://github.com/sqlpage/SQLPage/issues/516
SET x = 'It works !';
exec sp_executesql N'SELECT @p as actual, @exp as expected', N'@p varchar(100), @exp varchar(100)', @p=$x, @exp='It works !';
