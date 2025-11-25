select 'It works !' as expected_contains,
    sqlpage.exec('echo', 'It', $thisisnull, 'works', '!') as actual;