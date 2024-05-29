SELECT 'cookie' AS component,
	'lightdarkstatus' AS name,
	IIF(COALESCE(sqlpage.cookie('lightdarkstatus'),'') = '', 'dark', '') AS value;

SELECT 'redirect' AS component, sqlpage.header('referer') AS link;
