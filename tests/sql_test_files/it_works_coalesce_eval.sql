select 'https://example.com' as expected, sqlpage.link(coalesce($i_do_not_exist, 'https://example.com')) as actual;
