select '{"x":"1"}' as expected,
    sqlpage.variables('get') as actual;