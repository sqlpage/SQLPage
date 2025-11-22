FROM --platform=$BUILDPLATFORM rust:1.90-slim AS builder

WORKDIR /usr/src/sqlpage
ARG TARGETARCH
ARG BUILDARCH

COPY scripts/ /usr/local/bin/
RUN cargo init .

RUN /usr/local/bin/setup-cross-compilation.sh "$TARGETARCH" "$BUILDARCH"

COPY Cargo.toml Cargo.lock ./
RUN /usr/local/bin/build-dependencies.sh

COPY . .
RUN /usr/local/bin/build-project.sh

FROM busybox:glibc
RUN addgroup --gid 1000 --system sqlpage && \
    adduser --uid 1000 --system --no-create-home --ingroup sqlpage sqlpage && \
    mkdir -p /etc/sqlpage && \
    touch /etc/sqlpage/sqlpage.db && \
    chown -R sqlpage:sqlpage /etc/sqlpage/sqlpage.db
ENV SQLPAGE_WEB_ROOT=/var/www
ENV SQLPAGE_CONFIGURATION_DIRECTORY=/etc/sqlpage
WORKDIR /var/www
COPY --from=builder /usr/src/sqlpage/sqlpage.bin /usr/local/bin/sqlpage
# Provide runtime helper libs in system lib directory for the glibc busybox base
COPY --from=builder /tmp/sqlpage-libs/* /lib/
USER sqlpage
COPY --from=builder --chown=sqlpage:sqlpage /usr/src/sqlpage/sqlpage/sqlpage.db sqlpage/sqlpage.db
EXPOSE 8080
CMD ["/usr/local/bin/sqlpage"]