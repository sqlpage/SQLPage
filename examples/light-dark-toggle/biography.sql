SELECT 'dynamic' AS component,
	sqlpage.run_sql('shell.sql')
	AS properties;

SELECT 'text' AS component,
	'Biography' AS title;
SELECT 'Morbi fermentum porttitor bibendum. Vivamus eu tempus purus. Sed ligula risus, consectetur in ligula eu, lobortis sollicitudin tellus. Lorem ipsum dolor sit amet, consectetur adipiscing elit. Proin purus justo, lacinia in velit sed, fringilla imperdiet neque. Suspendisse iaculis lacus metus, at imperdiet justo rutrum nec. Duis accumsan fermentum nisi quis ornare. Aenean at placerat quam, quis gravida diam. Sed sollicitudin justo sit amet mattis eleifend. Vestibulum eget porttitor quam.' AS contents;
