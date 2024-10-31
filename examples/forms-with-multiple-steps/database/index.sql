-- create a new empty partially_filled_users row, returning its id
insert into partially_filled_users default values
returning
  'redirect' as component,
  'step_1.sql?id=' || id as link;
