-- This shell goes to every page

SELECT 'shell' AS component,
	'LightDark' AS title,
	sqlpage.cookie('lightdarkstatus') AS theme,
	'/' AS link,
	'[
		{"title":"Categories",
			"submenu": [
				{"title":"Home","link":"/"},
				{"title":"Presentation","link":"/presentation.sql"},
				{"title":"Biography","link":"/biography.sql"},
				{"title":"Code of conduct","link":"/codeconduct.sql"}
			]},
		{"title":"☀","link":"/toggle.sql"}
	]' AS menu_item,
	'sqlpage ' || sqlpage.version()
	AS footer;
