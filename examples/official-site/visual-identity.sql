select 'http_header' as component,
    'public, max-age=600, stale-while-revalidate=3600, stale-if-error=86400' as "Cache-Control";

select 'dynamic' as component, json_patch(json_extract(properties, '$[0]'), json_object(
    'title', 'Visual Identity - SQLPage',
    'css', '/assets/highlightjs-and-tabler-theme.css',
    'theme', 'dark'
)) as properties
FROM example WHERE component = 'shell' LIMIT 1;

select 'text' as component, 'Visual Identity Guide' as title, '
This guide defines the visual identity of SQLPage for consistent brand representation.
' as contents_md;

select 'text' as component, 'Personality' as title, '
**Playful yet professional**: Approachable, innovative, confident, energetic, reliable, creative.
' as contents_md;

select 'text' as component, 'Logo' as title, '
Primary logo: `/assets/icon.webp`

**Usage**:
- Minimum size: 48px height
- Clear space: 50% of logo height
- Do not distort, rotate, or modify
- Works on dark and light backgrounds
' as contents_md;

select 'html' as component, '
<div style="display: flex; justify-content: center; align-items: center; padding: 2rem 0;">
    <img src="/assets/icon.webp" alt="SQLPage Logo" style="max-width: 200px; height: auto;" />
</div>
' as html;

select 'button' as component;
select 
    'Download Logo' as title,
    '/assets/icon.webp' as link,
    'icon.webp' as download,
    'blue' as color,
    'download' as icon;

select 'text' as component, 'Colors' as title, '
Color palette extracted directly from the logo and design system.
' as contents_md;

select 'color_swatch' as component;
select 
    'Primary Cyan' as name,
    '#37E5EF' as hex,
    'Main logo color - bright cyan' as description;
select 
    'Teal Accent' as name,
    '#2A9FAF' as hex,
    'Secondary teal from logo' as description;
select 
    'Dark Navy' as name,
    '#090D19' as hex,
    'Logo background - dark navy' as description;
select 
    'Medium Blue' as name,
    '#27314C' as hex,
    'Medium blue from logo' as description;
select 
    'Blue Gray' as name,
    '#304960' as hex,
    'Blue-gray from logo' as description;
select 
    'Neutral Gray' as name,
    '#4B4E5C' as hex,
    'Neutral gray from logo' as description;
select 
    'Light Gray' as name,
    '#9FA4AE' as hex,
    'Light gray from logo' as description;
select 
    'Primary Background' as name,
    '#0a0f1a' as hex,
    'Dark theme foundation' as description;
select 
    'Primary Text' as name,
    '#f7f7f7' as hex,
    'Main text color' as description;
select 
    'White' as name,
    '#ffffff' as hex,
    'Headings and emphasis' as description;

select 'text' as component, 'Gradient' as title, '
Primary gradient flows from primary cyan (#37E5EF) to teal accent (#2A9FAF).

Use for buttons, highlights, and important elements.
' as contents_md;

select 'text' as component, 'Typography' as title, '
**Primary Font**: Inter

Use Inter for all digital materials, websites, and presentations. Inter is a modern, highly legible sans-serif typeface designed specifically for user interfaces.

**Font Source**: [Google Fonts - Inter](https://fonts.google.com/specimen/Inter)

**Fallback Font Stack**: Inter, -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, "Helvetica Neue", Arial, sans-serif

If Inter is not available, use the fallback stack in order.
' as contents_md;

select 'typography_sample' as component;
select 
    'Page Title' as title,
    'Inter, -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, "Helvetica Neue", Arial, sans-serif' as font_family,
    '64px' as font_size,
    '800' as font_weight,
    '1.1' as line_height,
    '#ffffff' as text_color,
    '-1px' as letter_spacing,
    'SQLPage Visual Identity' as sample_text,
    'Hero sections, main page titles, presentation title slides' as usage,
    'Bold, impactful text for maximum visual hierarchy' as description;

select 'typography_sample' as component;
select 
    'Section Heading' as title,
    'Inter, -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, "Helvetica Neue", Arial, sans-serif' as font_family,
    '56px' as font_size,
    '700' as font_weight,
    '1.2' as line_height,
    '#ffffff' as text_color,
    'normal' as letter_spacing,
    'Section Title' as sample_text,
    'Major section breaks, chapter headings, presentation section slides' as usage,
    'Strong but slightly less prominent than page titles' as description;

select 'typography_sample' as component;
select 
    'Subsection Heading' as title,
    'Inter, -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, "Helvetica Neue", Arial, sans-serif' as font_family,
    '40px' as font_size,
    '600' as font_weight,
    '1.3' as line_height,
    '#ffffff' as text_color,
    'normal' as letter_spacing,
    'Subsection Heading' as sample_text,
    'Card titles, subsection headers, content slide titles' as usage,
    'Clear hierarchy for organizing content' as description;

select 'typography_sample' as component;
select 
    'Body Text' as title,
    'Inter, -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, "Helvetica Neue", Arial, sans-serif' as font_family,
    '16px' as font_size,
    '400' as font_weight,
    '1.6' as line_height,
    '#f7f7f7' as text_color,
    'normal' as letter_spacing,
    'This is body text used for paragraphs, descriptions, and main content. It should be comfortable to read with adequate spacing between lines.' as sample_text,
    'Paragraphs, descriptions, main content, presentation body text' as usage,
    'Standard reading size with comfortable line spacing' as description;

select 'typography_sample' as component;
select 
    'Small Text' as title,
    'Inter, -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, "Helvetica Neue", Arial, sans-serif' as font_family,
    '14px' as font_size,
    '400' as font_weight,
    '1.5' as line_height,
    '#999999' as text_color,
    'normal' as letter_spacing,
    'Small text for captions and secondary information' as sample_text,
    'Captions, metadata, footnotes, fine print' as usage,
    'Supporting information that should not compete with main content' as description;

select 'text' as component, 'Spacing' as title, '
**Base unit**: 8 pixels

**Spacing scale**:
- Extra small: 8 pixels
- Small: 16 pixels
- Medium: 24 pixels
- Large: 32 pixels
- Extra large: 48 pixels
- XXL: 64 pixels

**Container**: Maximum width 1000 pixels, padding 40 pixels
' as contents_md;

select 'text' as component, 'Dark Environments' as title, '
For digital displays, presentations, and screens.
' as contents_md;

select 'text' as component, 'Background Colors' as title, '
- **Primary background**: #0a0f1a (deep navy blue)
- **Secondary background**: #0f1426 (slightly lighter navy)
- Use gradients with primary cyan (#37E5EF) and teal (#2A9FAF) for visual interest
' as contents_md;

select 'text' as component, 'Text Colors' as title, '
- **Primary text**: #f7f7f7 (almost white) - for main content
- **Secondary text**: #999999 (medium gray) - for supporting information
- **Headings**: #ffffff (pure white) - for maximum emphasis
- **Links**: #7db3e8 (bright blue) - for interactive elements
' as contents_md;

select 'text' as component, 'Contrast Guidelines' as title, '
- All text must meet WCAG AA contrast requirements (minimum 4.5:1 for normal text, 3:1 for large text)
- Primary text (#f7f7f7) on primary background (#0a0f1a) meets accessibility standards
- Use white (#ffffff) only for headings and emphasis
- Test all color combinations before finalizing designs
' as contents_md;

select 'text' as component, 'Light Environments' as title, '
For print materials, light-themed websites, and bright displays.
' as contents_md;

select 'text' as component, 'Background Colors' as title, '
- **Primary background**: #ffffff (white) or #f8f9fa (off-white)
- **Secondary background**: #f1f3f5 (light gray)
- Use subtle gradients or solid light colors
- Avoid pure white backgrounds in print to reduce glare
' as contents_md;

select 'text' as component, 'Text Colors' as title, '
- **Primary text**: #1a1a1a (near black) or #212529 (dark gray) - for main content
- **Secondary text**: #6c757d (medium gray) - for supporting information
- **Headings**: #000000 (black) or #0a0f1a (dark navy) - for emphasis
- **Links**: #2A9FAF (teal) or #37E5EF (cyan) - maintain brand colors
' as contents_md;

select 'text' as component, 'Logo Usage in Light Environments' as title, '
- Logo works on both light and dark backgrounds
- On light backgrounds, ensure sufficient contrast
- Consider using a darker version or adding a subtle shadow if needed
- Test logo visibility on various light backgrounds
' as contents_md;

select 'text' as component, 'Print Guidelines' as title, '
- Use CMYK color mode for print materials
- Convert hex colors to CMYK equivalents
- Primary cyan (#37E5EF) prints as: C: 76%, M: 0%, Y: 0%, K: 6%
- Teal accent (#2A9FAF) prints as: C: 76%, M: 9%, Y: 0%, K: 31%
- Test print samples to ensure color accuracy
- Use off-white paper (#f8f9fa equivalent) to reduce eye strain
- Minimum font size for print: 10 points (13 pixels)
- Ensure all text meets print contrast requirements
' as contents_md;

select 'text' as component, 'Presentations' as title, '
**Background**: Dark theme #0a0f1a with gradient overlays

**Typography**: 
- Title slide: Large bold text with gradient effect
- Body: Minimum readable size for your audience
- Code: Monospace font, minimum readable size

**Logo**: 
- Title slide: Large, centered
- Content slides: Small, bottom-right corner

**Colors**: Use brand cyan/teal gradients (#37E5EF to #2A9FAF) for highlights. Maintain high contrast for readability.
' as contents_md;

select 'text' as component, 'Resources' as title, '
- Logo: `/assets/icon.webp`
- CSS Theme: `/assets/highlightjs-and-tabler-theme.css`
- [Components Documentation](/component.sql)
- [GitHub Discussions](https://github.com/sqlpage/SQLPage/discussions)
' as contents_md;
