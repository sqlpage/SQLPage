select 
    'card' as component,
    2      as columns;
select 
    'A card with a Markdown description' as title,
    'This is a card with a **Markdown** description. 

This is useful if you want to display a lot of text in the card, with many options for formatting, such as 
 - **bold**, 
 - *italics*, 
 - [links](index.sql), 
 - etc.' as description_md;
select 
    'A card with lazy-loaded chart' as title,
	'/chart-example.sql?_sqlpage_embed' as embed;
select 
    'A card with lazy-loaded map' as title,
	'/map-example.sql?_sqlpage_embed' as embed;
select 
    'A card with lazy-loaded table' as title,
	'/table-example.sql?_sqlpage_embed' as embed;
