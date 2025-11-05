select 'shell' as component, 'dark' as theme, '[View source on Github](https://github.com/sqlpage/SQLPage/blob/main/examples/official-site/examples/form.sql)' as footer;

SELECT 'form' AS component, 'Complete Input Types Reference' AS title, '/examples/show_variables.sql' as action;

SELECT 'header' AS type, 'Text Input Types' AS label;

SELECT 'username' AS name, 'text' AS type, 'Enter your username' AS placeholder,
    '**Text** - Default single-line text input. Use for short text like names, usernames, titles. Supports `minlength`, `maxlength`, `pattern` for validation.' AS description_md;

SELECT 'password' AS name, 'password' AS type, '^(?=.*[A-Za-z])(?=.*\d)[A-Za-z\d]{8,}$' AS pattern,
    '**Password** - Masked text input that hides characters. Use for passwords and sensitive data. Combine with `pattern` attribute for password strength requirements.' AS description_md;

SELECT 'search_query' AS name, 'search' AS type, 'Search...' AS placeholder,
    '**Search** - Search input field, may display a clear button. Use for search boxes. Mobile browsers may show optimized keyboard.' AS description_md;

SELECT 'bio' AS name, 'textarea' AS type, 5 AS rows, 'Tell us about yourself...' AS placeholder,
    '**Textarea** (SQLPage custom) - Multi-line text input. Use for long text like comments, descriptions, articles. Set `rows` to control initial height.' AS description_md;

SELECT 'header' AS type, 'Numeric Input Types' AS label;

SELECT 'age' AS name, 'number' AS type, 0 AS min, 120 AS max, 1 AS step,
    '**Number** - Numeric input with up/down arrows. Use for quantities, ages, counts. Supports `min`, `max`, `step`. Mobile shows numeric keyboard.' AS description_md;

SELECT 'price' AS name, 'number' AS type, 0.01 AS step, '$' AS prefix,
    '**Number with decimals** - Set `step="0.01"` for currency. Use `prefix`/`suffix` for units. Great for prices, measurements, percentages.' AS description_md;

SELECT 'volume' AS name, 'range' AS type, 0 AS min, 100 AS max, 50 AS value, 1 AS step,
    '**Range** - Slider control for selecting a value. Use for volume, brightness, ratings, or any bounded numeric value where precision isn''t critical.' AS description_md;

SELECT 'header' AS type, 'Date and Time Types' AS label;

SELECT 'birth_date' AS name, 'date' AS type,
    '**Date** - Date picker (year, month, day). Use for birthdays, deadlines, event dates. Most browsers show a calendar widget. Supports `min` and `max` for date ranges.' AS description_md;

SELECT 'appointment_time' AS name, 'time' AS type,
    '**Time** - Time picker (hours and minutes). Use for appointment times, opening hours, alarms. Shows time selector in supported browsers.' AS description_md;

SELECT 'meeting_datetime' AS name, 'datetime-local' AS type,
    '**Datetime-local** - Date and time picker without timezone. Use for scheduling events, booking appointments, logging timestamps in local time.' AS description_md;

SELECT 'birth_month' AS name, 'month' AS type,
    '**Month** - Month and year picker. Use for credit card expiration dates, monthly reports, subscription periods.' AS description_md;

SELECT 'vacation_week' AS name, 'week' AS type,
    '**Week** - Week and year picker. Use for week-based scheduling, timesheet entry, weekly reports.' AS description_md;

SELECT 'header' AS type, 'Contact Information Types' AS label;

SELECT 'user_email' AS name, 'email' AS type, 'user@example.com' AS placeholder,
    '**Email** - Email address input with built-in validation. Use for email fields. Browser validates format automatically. Mobile shows @ key on keyboard.' AS description_md;

SELECT 'phone' AS name, 'tel' AS type, '+1 (555) 123-4567' AS placeholder,
    '**Tel** - Telephone number input. Use for phone numbers. Mobile browsers show numeric keyboard with phone symbols. No automatic validation - use `pattern` if needed.' AS description_md;

SELECT 'website' AS name, 'url' AS type, 'https://example.com' AS placeholder,
    '**URL** - URL input with validation. Use for website addresses, links. Browser validates URL format. Mobile may show .com key on keyboard.' AS description_md;

SELECT 'header' AS type, 'Selection Types' AS label;

SELECT 'country' AS name, 'select' AS type, 
    '[{"label": "United States", "value": "US"}, {"label": "Canada", "value": "CA"}, {"label": "United Kingdom", "value": "GB"}]' AS options,
    '**Select** (SQLPage custom) - Dropdown menu. Use for single choice from many options. Add `multiple` for multi-select. Use `searchable` for long lists. Set `dropdown` for enhanced UI.' AS description_md;

SELECT 'gender' AS name, 'radio' AS type, 'Male' AS value, 'Male' AS label,
    '**Radio** - Radio button for mutually exclusive choices. Create multiple rows with same `name` for a radio group. One option can be selected. Use for 2-5 options.' AS description_md;

SELECT 'gender' AS name, 'radio' AS type, 'Female' AS value, 'Female' AS label;

SELECT 'gender' AS name, 'radio' AS type, 'Other' AS value, 'Other' AS label;

SELECT 'interests' AS name, 'checkbox' AS type, 'Technology' AS value, 'Technology' AS label,
    '**Checkbox** - Checkbox for multiple selections. Each checkbox is independent. Use for yes/no questions or multiple selections from a list.' AS description_md;

SELECT 'terms' AS name, 'checkbox' AS type, TRUE AS required, 'I accept the terms and conditions' AS label,
    '**Checkbox (required)** - Use `required` to make acceptance mandatory. Common for terms of service, privacy policies, consent forms.' AS description_md;

SELECT 'notifications' AS name, 'switch' AS type, 'Enable email notifications' AS label, TRUE AS checked,
    '**Switch** (SQLPage custom) - Toggle switch, styled checkbox alternative. Use for on/off settings, feature toggles, preferences. More intuitive than checkboxes for boolean settings.' AS description_md;

SELECT 'header' AS type, 'File and Media Types' AS label;

SELECT 'profile_picture' AS name, 'file' AS type, 'image/*' AS accept,
    '**File** - File upload control. Use `accept` to limit file types (image/\*, .pdf, .doc). Use `multiple` to allow multiple files. Automatically sets form enctype to multipart/form-data.' AS description_md;

SELECT 'documents[]' AS name, 'Documents' as label, 'file' AS type, '.pdf,.doc,.docx' AS accept, TRUE AS multiple,
    '**File (multiple)** - Allow multiple file uploads with `multiple` attribute. Specify exact extensions or MIME types in `accept`.' AS description_md;

SELECT 'favorite_color' AS name, 'color' AS type, '#3b82f6' AS value,
    '**Color** - Color picker. Use for theme customization, design settings, highlighting preferences. Returns hex color code (#RRGGBB).' AS description_md;

SELECT 'user_id' AS name, 'hidden' AS type, '12345' AS value,
    '**Hidden** - Hidden input, not visible to users. Use for IDs, tokens, state information that needs to be submitted but not displayed or edited.' AS description_md;
