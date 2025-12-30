select
       sqlpage.variables() as all_vars,
       sqlpage.variables('get') as get_vars,
       sqlpage.variables('post') as post_vars,
       sqlpage.variables('set') as set_vars;

set my_set_var = 'set_value';
set common = 'set_common_value';

select
       sqlpage.variables() as all_vars,
       sqlpage.variables('get') as get_vars,
       sqlpage.variables('post') as post_vars,
       sqlpage.variables('set') as set_vars;
