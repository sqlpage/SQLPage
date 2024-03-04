select 'http_header' as component, 'application/rss+xml' as "Content-Type";
select 'shell-empty' as component;
select
	'rss' as component,
	'SQLPage blog' as title,
	'https://sql.ophir.dev/blog.sql' as link,
	'latest news about SQLpage' as description;
select
	'Hello everyone !' as title,
	'https://sql.ophir.dev/blog.sql?post=Come%20see%20me%20build%20twitter%20live%20on%20stage%20in%20Prague' as link,
	'If some of you european SQLPagers are around Prague this december, I will be giving a talk about SQLPage at pgconf.eu on December 14th.' as description;
select
	'3 solutions to the 3 layer problem' as title,
	'https://sql.ophir.dev/blog.sql?post=3%20solutions%20to%20the%203%20layer%20problem' as link,
	'Some interesting questions emerged from the article Repeating yourself.' as description,
	'Mon, 04 Dec 2023 00:00:00 GMT' as pubdate;
