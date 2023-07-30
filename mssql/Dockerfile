ARG VERSION=2019-latest
FROM mcr.microsoft.com/mssql/server:${VERSION}

# Create a config directory
RUN mkdir -p /usr/config
WORKDIR /usr/config

# Bundle config source
COPY entrypoint.sh /usr/config/entrypoint.sh
COPY configure-db.sh /usr/config/configure-db.sh
COPY setup.sql /usr/config/setup.sql

# Grant permissions for to our scripts to be executable
USER root
RUN chmod +x /usr/config/entrypoint.sh
RUN chmod +x /usr/config/configure-db.sh
RUN chown 10001 /usr/config/entrypoint.sh
RUN chown 10001 /usr/config/configure-db.sh
USER 10001

ENV SA_PASSWORD="Password123!"
ENV ACCEPT_EULA="Y"

ENTRYPOINT ["/usr/config/entrypoint.sh"]
