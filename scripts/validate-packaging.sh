#!/bin/bash
set -euo pipefail

# Validate packaging configuration files
# Run this before committing changes to packaging files

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

ERRORS=0
WARNINGS=0

log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[✓]${NC} $1"
}

log_warn() {
    echo -e "${YELLOW}[⚠]${NC} $1"
    ((WARNINGS++))
}

log_error() {
    echo -e "${RED}[✗]${NC} $1"
    ((ERRORS++))
}

cd "$PROJECT_ROOT"

log_info "Validating SQLPage packaging configuration..."
echo

# Check Debian files
log_info "Checking Debian package files..."
if [ -d "debian" ]; then
    REQUIRED_DEB_FILES=(
        "debian/control"
        "debian/rules"
        "debian/changelog"
        "debian/compat"
        "debian/copyright"
    )
    
    for file in "${REQUIRED_DEB_FILES[@]}"; do
        if [ -f "$file" ]; then
            log_success "$file exists"
        else
            log_error "$file is missing"
        fi
    done
    
    # Check if rules is executable
    if [ -x "debian/rules" ]; then
        log_success "debian/rules is executable"
    else
        log_warn "debian/rules is not executable (run: chmod +x debian/rules)"
    fi
    
    # Check postinst/postrm are executable if they exist
    for script in postinst postrm preinst prerm; do
        if [ -f "debian/$script" ]; then
            if [ -x "debian/$script" ]; then
                log_success "debian/$script is executable"
            else
                log_warn "debian/$script exists but is not executable"
            fi
        fi
    done
    
    # Validate changelog format
    if [ -f "debian/changelog" ]; then
        if head -1 debian/changelog | grep -qE '^[a-z0-9][a-z0-9+.-]+ \([^)]+\) [a-z]+; urgency='; then
            log_success "debian/changelog has valid format"
        else
            log_error "debian/changelog has invalid format"
        fi
    fi
else
    log_error "debian/ directory not found"
fi

echo

# Check RPM files
log_info "Checking RPM package files..."
if [ -d "rpm" ]; then
    if [ -f "rpm/sqlpage.spec" ]; then
        log_success "rpm/sqlpage.spec exists"
        
        # Check for required sections
        REQUIRED_SECTIONS=(
            "%description"
            "%prep"
            "%build"
            "%install"
            "%files"
        )
        
        for section in "${REQUIRED_SECTIONS[@]}"; do
            if grep -q "^$section" rpm/sqlpage.spec; then
                log_success "rpm/sqlpage.spec contains $section"
            else
                log_error "rpm/sqlpage.spec missing $section"
            fi
        done
    else
        log_error "rpm/sqlpage.spec not found"
    fi
else
    log_error "rpm/ directory not found"
fi

echo

# Check scripts
log_info "Checking build scripts..."
REQUIRED_SCRIPTS=(
    "scripts/build-deb.sh"
    "scripts/build-rpm.sh"
    "scripts/test-packages.sh"
)

for script in "${REQUIRED_SCRIPTS[@]}"; do
    if [ -f "$script" ]; then
        if [ -x "$script" ]; then
            log_success "$script exists and is executable"
        else
            log_warn "$script exists but is not executable"
        fi
    else
        log_error "$script not found"
    fi
done

echo

# Check CI/CD workflows
log_info "Checking CI/CD workflows..."
if [ -f ".github/workflows/packages.yml" ]; then
    log_success ".github/workflows/packages.yml exists"
    
    # Check for required jobs
    REQUIRED_JOBS=(
        "build-deb"
        "build-rpm"
        "test-deb-debian"
        "test-rpm-fedora"
    )
    
    for job in "${REQUIRED_JOBS[@]}"; do
        if grep -q "$job:" .github/workflows/packages.yml; then
            log_success "packages.yml contains job: $job"
        else
            log_error "packages.yml missing job: $job"
        fi
    done
else
    log_error ".github/workflows/packages.yml not found"
fi

echo

# Check version consistency
log_info "Checking version consistency..."
CARGO_VERSION=$(grep '^version = ' Cargo.toml | head -1 | sed 's/version = "\(.*\)"/\1/')
log_info "Cargo.toml version: $CARGO_VERSION"

if [ -f "debian/changelog" ]; then
    DEB_VERSION=$(head -1 debian/changelog | sed 's/.*(\([^)]*\)).*/\1/')
    log_info "debian/changelog version: $DEB_VERSION"
fi

if [ -f "rpm/sqlpage.spec" ]; then
    RPM_VERSION=$(grep '^Version:' rpm/sqlpage.spec | awk '{print $2}')
    log_info "rpm/sqlpage.spec version: $RPM_VERSION"
fi

echo

# Check documentation
log_info "Checking documentation..."
REQUIRED_DOCS=(
    "PACKAGING.md"
    "scripts/README.md"
)

for doc in "${REQUIRED_DOCS[@]}"; do
    if [ -f "$doc" ]; then
        log_success "$doc exists"
    else
        log_warn "$doc not found"
    fi
done

echo

# Summary
echo "================================================"
if [ $ERRORS -eq 0 ] && [ $WARNINGS -eq 0 ]; then
    log_success "All validation checks passed!"
    exit 0
elif [ $ERRORS -eq 0 ]; then
    echo -e "${YELLOW}Validation completed with $WARNINGS warning(s)${NC}"
    exit 0
else
    echo -e "${RED}Validation failed with $ERRORS error(s) and $WARNINGS warning(s)${NC}"
    exit 1
fi
