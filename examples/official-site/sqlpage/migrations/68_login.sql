INSERT INTO component(name, icon, description, introduced_in_version) VALUES
    ('login', 'password-user', '
The login component is an authentication form for users of an application. 

It allows the entry of a user account consisting of a username and a password. 

It offers additional features such as the ability to request session persistence or to reset the password.', '0.39.0');

INSERT INTO parameter(component, name, description, type, top_level, optional) SELECT 'login', * FROM (VALUES
    ('title','Title of the authentication form.','TEXT',TRUE,TRUE),
    ('enctype','Form data encoding.','TEXT',TRUE,TRUE),
    ('action','An optional link to a target page that will handle the results of the form. ','TEXT',TRUE,TRUE),
    ('username','User account identifier.','TEXT',TRUE,FALSE),
    ('password','User password.','TEXT',TRUE,FALSE),
    ('username_icon','Icon to display on the left side of the input field, on the same line.','ICON',TRUE,TRUE),
    ('password_icon','Icon to display on the left side of the input field, on the same line.','ICON',TRUE,TRUE),
    ('image','The URL of an centered image displayed before the title.','URL',TRUE,TRUE),
    ('forgot_password_text','A text for the link allowing the user to reset their password. If the text is empty, the link is not displayed.','TEXT',TRUE,TRUE),
    ('forgot_password_link','The link to the page allowing the user to reset their password.','TEXT',TRUE,TRUE),
    ('remember_me_text','A text for the option allowing the user to request the preservation of their work session. The name of the field is remember. If the text is empty, the option is not displayed.','TEXT',TRUE,TRUE),
    ('footer','A text placed at the bottom of the authentication form.','TEXT',TRUE,TRUE),
    ('footer_md','A markdown text placed at the bottom of the authentication form. Useful for creating links to other pages (creating a new account, contacting technical support, etc.).','TEXT',TRUE,TRUE),
    ('validate','The text to display in the button at the bottom of the form that submits the values.','TEXT',TRUE,FALSE),
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
    );
