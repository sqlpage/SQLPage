## Download a PDF file from the server

Download a PDF file with the proper content type so PDF readers recognize it.
Uses [`sqlpage.read_file_as_data_url(file_path)`](/functions?function=read_file_as_data_url) to read the file from the server.

```sql
select
    'download' as component,
    'report.pdf' as filename,
    sqlpage.read_file_as_data_url('report.pdf') as data_url;
```
