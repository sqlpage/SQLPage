set $person = 'John' || ' ' || 'Doe';
select 'text' as component, 'Hello ' || $person || ' !' as contents;
select 'text' as component, 'How are you ' || $person || ' ?' as contents;