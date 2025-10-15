# Pull Request

## Description
Brief description of the changes made.

## Type of Change
- [ ] Bug fix (non-breaking change which fixes an issue)
- [ ] New feature (non-breaking change which adds functionality)
- [ ] Breaking change (fix or feature that would cause existing functionality to not work as expected)
- [ ] Documentation update
- [ ] Code refactoring
- [ ] Performance improvement

## Documentation Checklist
If this PR affects any of the following, please ensure documentation is updated:

### Components
- [ ] New component added → Documentation added to `docs/components/`
- [ ] Component modified → Documentation updated in `docs/components/`
- [ ] Component removed → Documentation removed from `docs/components/`

### Functions
- [ ] New function added → Documentation added to `docs/functions/`
- [ ] Function modified → Documentation updated in `docs/functions/`
- [ ] Function removed → Documentation removed from `docs/functions/`

### Configuration
- [ ] New configuration option → Documentation added to `docs/configuration/`
- [ ] Configuration modified → Documentation updated in `docs/configuration/`

### Guides
- [ ] New guide needed → Guide added to `docs/guides/`
- [ ] Existing guide needs update → Guide updated in `docs/guides/`

### API Changes
- [ ] Breaking API change → Migration guide added to `docs/guides/`
- [ ] New API endpoint → Documentation added to appropriate section

## Testing
- [ ] Unit tests added/updated
- [ ] Integration tests added/updated
- [ ] Manual testing completed
- [ ] Documentation validation passes (`cargo script scripts/validate-docs.rs --dep walkdir --dep regex`)
- [ ] SQL syntax check passes (`cargo script scripts/check-sql.rs --dep regex --dep walkdir`)

## Documentation Validation
Before submitting, please run:

```bash
# Validate documentation structure
cargo script scripts/validate-docs.rs --dep walkdir --dep regex

# Check SQL syntax in documentation
cargo script scripts/check-sql.rs --dep regex --dep walkdir

# Check for stale documentation
cargo script scripts/check-stale-docs.rs --dep walkdir --dep regex

# Build documentation database
cargo script scripts/build-simple-db.rs --dep "rusqlite={version=\"0.31\", features=[\"bundled\"]}" --dep walkdir --dep regex
```

## Screenshots (if applicable)
Add screenshots to help explain your changes.

## Additional Notes
Any additional information that reviewers should know.