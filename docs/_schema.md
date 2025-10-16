# SQLPage Documentation Schema

This document defines the schema and authoring rules for SQLPage documentation. All documentation is authored in Markdown with minimal YAML frontmatter, avoiding duplication of data that can be derived from file paths.

## Core Principles

- **No Duplication**: Names, slugs, and dates are derived from file paths, not duplicated in frontmatter
- **Markdown-First**: Content is primarily in Markdown with structured sections
- **SQLite-Backed**: All documentation is compiled into a single `docs.sqlite` database
- **Validation**: All content is validated against this schema

## Directory Structure

```
docs/
├── _schema.md                    # This file
├── components/                   # Component documentation
│   └── {component}.md
├── functions/                    # Function documentation
│   └── {function}.md
├── guides/                       # User guides and tutorials
│   └── {topic}/index.md or {topic}.md
├── blog/                         # Blog posts
│   └── YYYY-MM-DD-{slug}.md
├── configuration/                # Configuration documentation
│   └── {topic}.md
└── architecture/                 # Architecture documentation
    └── {topic}.md
```

## Component Documentation (`docs/components/{component}.md`)

**Filename**: `{component}.md` (e.g., `form.md` → name=form)

**Frontmatter** (YAML):
- `icon` (optional): Tabler icon name
- `introduced_in_version` (optional): Version when component was introduced
- `deprecated_in_version` (optional): Version when component was deprecated
- `difficulty` (optional): `beginner` | `intermediate` | `advanced`

**Required Sections** (in order):
1. **Overview**: Brief description of the component
2. **When to Use**: Guidance on when to use this component
3. **Basic Usage**: SQL example showing basic usage
4. **Top-Level Parameters**: Markdown table of top-level parameters
5. **Row-Level Parameters**: Markdown table of row-level parameters
6. **Examples**: Additional SQL examples
7. **Related**: Links to related components, functions, or guides
8. **Changelog**: Version history and changes

**Parameter Tables Format**:
```markdown
| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| name | TEXT | Yes | - | Parameter description |
```

## Function Documentation (`docs/functions/{function}.md`)

**Filename**: `{function}.md` (e.g., `cookie.md` → name=cookie)

**Frontmatter** (YAML):
- `namespace` (optional): Default `sqlpage`
- `icon` (optional): Tabler icon name
- `return_type` (optional): Return type description
- `introduced_in_version` (optional): Version when function was introduced
- `deprecated_in_version` (optional): Version when function was deprecated
- `category` (optional): Function category
- `difficulty` (optional): `beginner` | `intermediate` | `advanced`

**Required Sections** (in order):
1. **Signature**: Fenced code block with function signature
2. **Description**: What the function does
3. **Parameters**: Table or H3 per parameter
4. **Return Value**: What the function returns
5. **Security Notes**: Security considerations (if relevant)
6. **Examples**: SQL examples showing usage
7. **Related**: Links to related functions, components, or guides

**Signature Format**:
```sql
function_name(param1 TYPE, param2 TYPE) -> RETURN_TYPE
```

## Guide Documentation (`docs/guides/{topic}/index.md` or `docs/guides/{topic}.md`)

**Filename**: `{topic}/index.md` or `{topic}.md` (slug derived from path)

**Frontmatter** (YAML):
- `title` (required): Guide title
- `difficulty` (optional): `beginner` | `intermediate` | `advanced`
- `estimated_time` (optional): Time estimate (e.g., "15 minutes")
- `introduced_in_version` (optional): Version when guide was introduced
- `categories` (optional): Array of categories
- `tags` (optional): Array of tags
- `prerequisites` (optional): Array of prerequisite guide slugs
- `next` (optional): Array of next guide slugs

**Content**: Free-form Markdown content

## Blog Documentation (`docs/blog/YYYY-MM-DD-{slug}.md`)

**Filename**: `YYYY-MM-DD-{slug}.md` (date and slug derived from filename)

**Frontmatter** (YAML):
- `title` (required): Blog post title
- `author` (optional): Author name
- `tags` (optional): Array of tags
- `categories` (optional): Array of categories
- `featured` (optional): Boolean, default false
- `preview_image` (optional): URL to preview image
- `excerpt` (optional): Short excerpt for listings

**Content**: Free-form Markdown content

## Configuration Documentation (`docs/configuration/{topic}.md`)

**Filename**: `{topic}.md` (slug derived from path)

**Frontmatter** (YAML):
- `title` (required): Page title
- `introduced_in_version` (optional): Version when configuration was introduced
- `categories` (optional): Array of categories
- `tags` (optional): Array of tags

**Required Sections**:
- **Settings**: Markdown table of configuration settings (if applicable)

**Settings Table Format**:
```markdown
| Setting | Type | Required | Default | Description |
|---------|------|----------|---------|-------------|
| DATABASE_URL | TEXT | Yes | - | Database connection string |
```

## Architecture Documentation (`docs/architecture/{topic}.md`)

**Filename**: `{topic}.md` (slug derived from path)

**Frontmatter** (YAML):
- `title` (optional): Page title
- `tags` (optional): Array of tags
- `last_reviewed` (optional): ISO8601 date
- `last_updated` (optional): ISO8601 date

**Content**: Free-form Markdown content

## SQL Code Blocks

All SQL examples must use fenced code blocks with `sql` language identifier:

```sql
SELECT * FROM users WHERE active = 1;
```

## Validation Rules

1. **Required Fields**: All required frontmatter fields must be present
2. **Required Sections**: All required sections must be present in the correct order
3. **Version Format**: Version strings must follow semantic versioning (e.g., "0.1.0")
4. **No Duplicates**: No duplicate component/function names or guide slugs
5. **Internal Links**: All internal links must resolve to existing content
6. **SQL Syntax**: All SQL code blocks must be syntactically valid
7. **Table Format**: Parameter and settings tables must follow the specified format

## SQLite Schema

The documentation is compiled into a SQLite database with the following main tables:

- `components` - Component documentation
- `component_parameters` - Component parameters (top-level and row-level)
- `component_examples` - Component examples
- `functions` - Function documentation
- `function_parameters` - Function parameters
- `function_examples` - Function examples
- `guides` - Guide documentation
- `blog_posts` - Blog post documentation
- `configuration_pages` - Configuration documentation
- `configuration_settings` - Configuration settings
- `search_index` - Full-text search index

See the main specification for detailed table schemas.