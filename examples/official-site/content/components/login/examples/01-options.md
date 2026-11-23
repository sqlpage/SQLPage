---
rows:
  - component: login
    action: /examples/show_variables
    image: ../assets/icon.webp
    title: Please login to your account
    username: Username
    password: Password
    autofocus: false
    username_icon: user
    password_icon: lock
    forgot_password_text: Forgot your password?
    forgot_password_link: reset_password.sql
    remember_me_text: Remember me
    footer_md: Don't have an account? [Register here](register.sql)
    validate: Sign in
---

Using the main options of the login component

When the user clicks the "Sign in" button, the form is submitted to the `/examples/show_variables.sql` page.
There, you will have access to the variables:
 - `:username`: the username entered by the user
 - `:password`: the password entered by the user
 - `:remember`: the string "on" if the checkbox was checked, or NULL if it was not checked
