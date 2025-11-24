-- "<!DOCTYPE html><body>It works !</body>" percent encoded
select 'download' as component, 'data:text/html,%3C!DOCTYPE%20html%3E%3Cbody%3EIt%20works%20!%3C%2Fbody%3E' as data_url;