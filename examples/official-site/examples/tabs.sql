create table if not exists tab_example_cards as 
select 'Leaf' as title, 'Leaf_1_web' as img, 'f4' as prefix, 'green' as color union all
select 'Caterpillar', 'Caterpillar_of_box_tree_moth%2C_Germany_2019', 'a9', 'green' union all
select 'Butterfly', 'Common_brimstone_butterfly_(Gonepteryx_rhamni)_male', '6a', 'green' union all
select 'Flower', 'Red-poppy-flower_-_West_Virginia_-_ForestWander', 'fd', 'red' union all
select 'Bird', 'Summer_tanager_(Piranga_rubra)_male_Copan_3', 'dd', 'red' union all
select 'Medusa', 'Aurelia_aurita_2', '5c', 'blue';

select 'tab' as component, true as center;
select 'Show all cards' as title, 'All things are beautiful' as description, '?' as link, $tab is null as active;
select format('Show %s cards', color) as title,
       format('%s things are beautiful', color) as description,
       format('?tab=%s', color) as link,
       $tab=color as active,
       case $tab when color then color end as color -- only show the color when the tab is active
from tab_example_cards
group by color; 


select 'card' as component;
select title, 
    format('https://upload.wikimedia.org/wikipedia/commons/thumb/%s/%s/%s.jpg/640px-%s.jpg', substr(prefix,1,1), prefix, img, img) as top_image,
    color
from tab_example_cards
where $tab is null or $tab = color;

select 'text' as component, 'See [source code on GitHub](https://github.com/lovasoa/SQLpage/blob/main/examples/official-site/examples/tabs.sql)' as contents_md;