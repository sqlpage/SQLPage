INSERT INTO component(name, icon, description, introduced_in_version) VALUES
    ('login', 'password-user', '
The login component is an authentication form with numerous customization options. 
It offers the main functionalities for this type of form. 
The user can enter their username and password. 
There are many optional attributes such as the use of icons on input fields, the insertion of a link to a page to reset the password, an option for the application to maintain the user''s identity via a cookie. 
It is also possible to set the title of the form, display the company logo, or customize the appearance of the form submission button.

This component should be used in conjunction with other components such as [authentication](component.sql?component=authentication) and [cookie](component.sql?component=cookie). 
It does not implement any logic and simply collects the username and password to pass them to the code responsible for authentication.

A few things to know :
- The form uses the POST method to transmit information to the destination page,
- The user''s username and password are entered into fields with the names `username` and `password`, 
- To obtain the values of username and password, you must use the variables `:username` and `:password`,
- To know if the user wants their identity to be remembered, you must read the value of the variable `:remember`.
', '0.39.0');

INSERT INTO parameter(component, name, description, type, top_level, optional) SELECT 'login', * FROM (VALUES
    ('title','Title of the authentication form.','TEXT',TRUE,TRUE),
    ('enctype','Form data encoding.','TEXT',TRUE,TRUE),
    ('action','An optional link to a target page that will handle the results of the form. ','TEXT',TRUE,TRUE),
    ('error_message','An error message to display above the form, typically shown after a failed login attempt.','TEXT',TRUE,TRUE),
    ('username','Label and placeholder for the user account identifier text field.','TEXT',TRUE,FALSE),
    ('password','Label and placeholder for the password field.','TEXT',TRUE,FALSE),
    ('username_icon','Icon to display on the left side of the input field, on the same line.','ICON',TRUE,TRUE),
    ('password_icon','Icon to display on the left side of the input field, on the same line.','ICON',TRUE,TRUE),
    ('image','The URL of an centered image displayed before the title.','URL',TRUE,TRUE),
    ('forgot_password_text','A text for the link allowing the user to reset their password. If the text is empty, the link is not displayed.','TEXT',TRUE,TRUE),
    ('forgot_password_link','The link to the page allowing the user to reset their password.','TEXT',TRUE,TRUE),
    ('remember_me_text','A text for the option allowing the user to request the preservation of their identity. If the text is empty, the option is not displayed.','TEXT',TRUE,TRUE),
    ('footer','A text placed at the bottom of the authentication form.','TEXT',TRUE,TRUE),
    ('footer_md','A markdown text placed at the bottom of the authentication form. Useful for creating links to other pages (creating a new account, contacting technical support, etc.).','TEXT',TRUE,TRUE),
    ('validate','The text to display in the button at the bottom of the form that submits the values.','TEXT',TRUE,TRUE),
    ('validate_color','The color of the button at the bottom of the form that submits the values. Omit this property to use the default color.','COLOR',TRUE,TRUE),
    ('validate_shape','The shape of the validation button.','TEXT',TRUE,TRUE),
    ('validate_outline','A color to outline the validation button.','COLOR',TRUE,TRUE),
    ('validate_size','The size of the validation button.','TEXT',TRUE,TRUE)
) x;

-- Insert example(s) for the component
INSERT INTO example(component, description, properties)
VALUES (
        'login',
        'Using the main options of the login component',
        JSON(
            '[
                {
                    "component": "login",
                    "action": "login.sql",
                    "image": "../assets/icon.webp",
                    "title": "Please login to your account",
                    "username": "Username",
                    "password": "Password",
                    "username_icon": "user",
                    "password_icon": "lock",
                    "forgot_password_text": "Forgot your password?",
                    "forgot_password_link": "reset_password.sql",
                    "remember_me_text": "Remember me",
                    "footer_md": "Don''t have an account? [Register here](register.sql)",
                    "validate": "Sign in"
                }
            ]'
        )
    ),
   ('login', 'Most basic login form', JSON('[{"component": "login"}]'));
