select 'alert' as component, 'Saved for this preview' as title,
    'Hello, ' || :name || '! Your form was submitted to SQLPage.' as description,
    'check' as icon, 'cyan' as color
where :name is not null;
select 'form' as component, 'Say hello' as title, 'Submit' as validate;
select 'name' as name, 'Your name' as label, true as required,
    'Ada' as placeholder;
select 'team' as name, 'Your team' as label, 'text' as type,
    'Small team, serious tools' as placeholder;
