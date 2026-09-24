SELECT
    CONCAT(
        'isnull=[', ISNULL($missing, 'hello'),
        '] coalesce=[', COALESCE($missing, 'hello'),
        '] len=', LEN(ISNULL($missing, 'hello')),
        '; date_is_date=', ISDATE(ISNULL($missing_date, GETDATE()))
    ) AS actual,
    'isnull=[hello] coalesce=[hello] len=5; date_is_date=1' AS expected;
