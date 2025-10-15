---
icon: forms
introduced_in_version: "0.1.0"
difficulty: beginner
---

# Form Component

## Overview

The `form` component creates an HTML form that can submit data to SQLPage for processing. It provides a simple way to collect user input and send it to your application.

## When to Use

Use the `form` component when you need to:
- Collect user input (text, numbers, selections)
- Submit data to your SQLPage application
- Create interactive user interfaces
- Handle user authentication or data entry

## Basic Usage

```sql
SELECT 'form' AS component, 'user_registration' AS name, 'POST' AS method, '/submit' AS action;
```

## Top-Level Parameters

| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| name | TEXT | Yes | - | Unique identifier for the form |
| method | TEXT | No | POST | HTTP method (GET or POST) |
| action | TEXT | No | - | URL to submit the form to |
| class | TEXT | No | - | CSS classes to apply |
| id | TEXT | No | - | HTML id attribute |

## Row-Level Parameters

| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| name | TEXT | Yes | - | Input field name |
| type | TEXT | No | text | Input type (text, email, password, etc.) |
| label | TEXT | No | - | Label for the input field |
| placeholder | TEXT | No | - | Placeholder text |
| required | BOOLEAN | No | false | Whether the field is required |
| value | TEXT | No | - | Default value |

## Examples

### Simple Contact Form

```sql
SELECT 'form' AS component, 'contact' AS name, 'POST' AS method, '/contact' AS action;

SELECT 'text' AS type, 'name' AS name, 'Your Name' AS label, true AS required;
SELECT 'email' AS type, 'email' AS name, 'Email Address' AS label, true AS required;
SELECT 'textarea' AS type, 'message' AS name, 'Message' AS label, true AS required;
SELECT 'submit' AS type, 'Send Message' AS value;
```

### User Registration Form

```sql
SELECT 'form' AS component, 'register' AS name, 'POST' AS method, '/register' AS action;

SELECT 'text' AS type, 'username' AS name, 'Username' AS label, true AS required;
SELECT 'email' AS type, 'email' AS name, 'Email' AS label, true AS required;
SELECT 'password' AS type, 'password' AS name, 'Password' AS label, true AS required;
SELECT 'password' AS type, 'confirm_password' AS name, 'Confirm Password' AS label, true AS required;
SELECT 'submit' AS type, 'Register' AS value;
```

## Related

- [Input Component](./input.md)
- [Button Component](./button.md)
- [Form Validation Guide](../guides/form-validation.md)
- [User Authentication Guide](../guides/user-authentication.md)

## Changelog

- **0.1.0**: Initial release with basic form functionality
- **0.2.0**: Added support for file uploads
- **0.3.0**: Added form validation attributes