ARG VERSION=2019-latest
FROM mcr.microsoft.com/mssql/server:${VERSION}

# Create a config directory
RUN mkdir -p /usr/config
WORKDIR /usr/config

# Bundle config source
COPY entrypoint.sh /usr/config/entrypoint.sh
COPY setup.sql /usr/config/setup.sql

# Grant permissions for to our scripts to be executable
USER root
RUN chmod +x /usr/config/entrypoint.sh
RUN chown 10001 /usr/config/entrypoint.sh
USER 10001

ENV SA_PASSWORD="Password123!"
ENV ACCEPT_EULA="Y"

HEALTHCHECK --interval=10s --timeout=3s --start-period=15s --retries=10 \
    CMD sqlcmd -S localhost -U root -P "Password123!" -Q "SELECT 1" || exit 1

ENTRYPOINT ["/usr/config/entrypoint.sh"]
