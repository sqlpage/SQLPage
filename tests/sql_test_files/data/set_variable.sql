set what_does_it_do = 'wo' || 'rks';
select 'It works !' as expected,
    'It ' || $what_does_it_do || ' !' as actual;
