FROM keycloak/keycloak:24.0

ADD  --chown=1000:0 https://github.com/jacekkow/keycloak-protocol-cas/releases/download/24.0.3/keycloak-protocol-cas-24.0.3.jar \
    /opt/keycloak/providers/keycloak-protocol-cas.jar

COPY ./keycloak-configuration.json /opt/keycloak/data/import/realm.json

CMD ["start-dev", "--import-realm", "--http-port", "8181"]