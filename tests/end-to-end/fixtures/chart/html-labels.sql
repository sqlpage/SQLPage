SELECT 'chart' AS component, 'test-chart' AS id, 'bar' AS type;
SELECT '<img src=x onerror="document.body.dataset.chartLabelExecuted=1">' AS series,
       '<b>Paid</b>' AS label, 28 AS value;
SELECT 'Baseline' AS series, '<b>Paid</b>' AS label, 20 AS value;
