FROM rustlang/rust:nightly-bullseye-slim as builder

WORKDIR /app

COPY . .

RUN set -eux; apt-get update && apt-get install -y libpq-dev \
    && rm -rf /var/lib/apt/lists/* \
    && cargo install --bins --path . --root ./artifacts

FROM debian:bullseye-slim

ARG GIT_REVISION
ENV APP_DEPLOYMENT_ID=${GIT_REVISION}

WORKDIR /app

COPY --from=builder /app/artifacts/bin/ /app/

RUN set -eux; apt-get update && apt-get install -y libpq-dev \
    && rm -rf /var/lib/apt/lists/*

CMD ["./api-sys"]