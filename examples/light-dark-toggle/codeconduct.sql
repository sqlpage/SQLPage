SELECT 'dynamic' AS component,
	sqlpage.run_sql('shell.sql')
	AS properties;

SELECT 'text' AS component,
	'Code Of Conduct' AS title;
SELECT 'Pellentesque sed consequat ligula. Ut fermentum elit diam, sit amet ullamcorper orci volutpat quis. Nunc nec ipsum eu nibh interdum interdum ut vitae neque. Sed ac hendrerit tortor, ac tincidunt nibh. Mauris vel tempor odio, quis varius lorem. In sed nibh placerat, fermentum nisl eget, dictum orci. Nullam sit amet ligula velit. Maecenas faucibus massa a orci pharetra, eu fringilla enim ornare. Vestibulum quis rutrum nisi. Pellentesque nec nulla eu tellus aliquet bibendum accumsan egestas dui. Phasellus arcu felis, dictum venenatis metus vel, consectetur finibus enim. Praesent tristique semper dolor, a mollis orci pharetra vel. Vivamus mattis, lectus blandit finibus euismod, magna justo ornare nisi, vel convallis nisi velit eu purus. Aliquam erat volutpat.' AS contents;
