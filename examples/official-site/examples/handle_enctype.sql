SET ":enctype" = CASE :percent_encoded IS NOT NULL OR :multipart_form_data IS NOT NULL
  WHEN TRUE THEN 'with ``' || COALESCE(:percent_encoded, :multipart_form_data) || '``'
  ELSE 'form'
END ||' encoding type'
SELECT 'text' AS component;
SELECT 'The following data was submitted '||:enctype||':
```
' || :data ||'
```' AS contents_md;