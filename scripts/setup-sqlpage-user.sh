#!/bin/bash
set -eux

# Create sqlpage user and group
addgroup --gid 1000 --system sqlpage
adduser --uid 1000 --system --no-create-home --ingroup sqlpage sqlpage

# Create and configure directories
mkdir -p /etc/sqlpage /var/lib/sqlpage /var/www
chown -R sqlpage:sqlpage /etc/sqlpage /var/lib/sqlpage /var/www
