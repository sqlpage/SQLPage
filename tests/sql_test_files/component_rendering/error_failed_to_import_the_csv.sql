-- https://github.com/sqlpage/SQLPage/issues/788
copy this_table_does_not_exist (csv) from 'recon_csv_file_input' DELIMITER '*' CSV;