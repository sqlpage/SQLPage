---
icon: password-user
since: 0.39.0
category: forms-and-feedback
summary: Display a sign-in form for a username and password.
top:
  - name: action
    type: TEXT
    description: 'An optional link to a target page that will handle the results of the form. '
  - name: autofocus
    type: BOOLEAN
    description: Focus the username field when the page opens. Defaults to true. Set to false when the form
      is one part of a larger page.
  - class
  - name: enctype
    type: TEXT
    description: Form data encoding.
  - name: error_message
    type: TEXT
    description: An error message to display above the form, typically shown after a failed login attempt.
  - name: error_message_md
    type: TEXT
    description: A markdown error message to display above the form, typically shown after a failed login attempt.
  - name: footer
    type: TEXT
    description: A text placed at the bottom of the authentication form. If both footer and footer_md are specified,
      footer takes precedence.
  - name: footer_md
    type: TEXT
    description: A markdown text placed at the bottom of the authentication form. Useful for creating links
      to other pages (creating a new account, contacting technical support, etc.).
  - name: forgot_password_link
    type: TEXT
    description: The link to the page allowing the user to reset their password.
  - name: forgot_password_text
    type: TEXT
    description: A text for the link allowing the user to reset their password. If the text is empty, the link
      is not displayed.
  - id
  - name: image
    type: URL
    description: The URL of an centered image displayed before the title.
  - name: password
    type: TEXT
    required: true
    description: Label and placeholder for the password field.
  - name: password_icon
    type: ICON
    description: Icon to display on the left side of the input field, on the same line.
  - name: remember_me_text
    type: TEXT
    description: A text for the option allowing the user to request the preservation of their identity. If
      the text is empty, the option is not displayed.
  - name: title
    type: TEXT
    description: Title of the authentication form.
  - name: username
    type: TEXT
    required: true
    description: Label and placeholder for the user account identifier text field.
  - name: username_icon
    type: ICON
    description: Icon to display on the left side of the input field, on the same line.
  - name: validate
    type: TEXT
    description: The text to display in the button at the bottom of the form that submits the values.
  - name: validate_color
    type: COLOR
    description: The color of the button at the bottom of the form that submits the values. Omit this property
      to use the default color.
  - name: validate_outline
    type: COLOR
    description: A color to outline the validation button.
  - name: validate_shape
    type: TEXT
    description: The shape of the validation button.
  - name: validate_size
    type: TEXT
    description: The size of the validation button.
---

The login component is an authentication form with numerous customization options.
It offers the main functionalities for this type of form.
The user can enter their username and password.
There are many optional attributes such as the use of icons on input fields, the insertion of a link to a page to reset the password, an option for the application to maintain the user's identity via a cookie.
It is also possible to set the title of the form, display the company logo, or customize the appearance of the form submission button.

This component should be used in conjunction with other components such as [authentication](component.sql?component=authentication) and [cookie](component.sql?component=cookie).
It does not implement any logic and simply collects the username and password to pass them to the code responsible for authentication.

A few things to know :
- The form uses the POST method to transmit information to the destination page,
- The user's username and password are entered into fields with the names `username` and `password`,
- To obtain the values of username and password, you must use the variables `:username` and `:password`,
- When you set the `remember_me_text` property, the variable `:remember` becomes available after form submission to check if the user checked the "remember me" checkbox.
