# Docker Build Scripts

This directory contains scripts used by the Dockerfile to build SQLPage with cross-compilation support.

## Scripts

- **`setup-cross-compilation.sh`**: Sets up the cross-compilation environment based on target and build architectures. Handles system dependencies, cross-compiler installation, and libgcc extraction for runtime.
- **`build-dependencies.sh`**: Builds only the project dependencies for Docker layer caching
- **`build-project.sh`**: Builds the final SQLPage binary

## Usage

These scripts are automatically copied and executed by the Dockerfile during the build process. They handle:

- Cross-compilation setup for different architectures (amd64, arm64, arm)
- System dependencies installation
- Cargo build configuration with appropriate linkers
- Library extraction for runtime

The scripts use temporary files in `/tmp/` to pass configuration between stages and export environment variables for use in subsequent build steps.
