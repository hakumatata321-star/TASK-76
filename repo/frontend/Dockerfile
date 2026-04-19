FROM rust:bookworm AS builder
RUN rustup target add wasm32-unknown-unknown
ARG TRUNK_VERSION=0.21.14
ARG TARGETARCH
RUN apt-get update && apt-get install -y --no-install-recommends curl ca-certificates && rm -rf /var/lib/apt/lists/*
RUN set -eux; \
    case "${TARGETARCH}" in \
      amd64) TRUNK_ARCH="x86_64-unknown-linux-gnu" ;; \
      arm64) TRUNK_ARCH="aarch64-unknown-linux-gnu" ;; \
      *) echo "Unsupported TARGETARCH: ${TARGETARCH}"; exit 1 ;; \
    esac; \
    curl -fL "https://github.com/trunk-rs/trunk/releases/download/v${TRUNK_VERSION}/trunk-${TRUNK_ARCH}.tar.gz" -o /tmp/trunk.tar.gz; \
    tar -xzf /tmp/trunk.tar.gz -C /usr/local/bin trunk; \
    chmod +x /usr/local/bin/trunk; \
    trunk --version
WORKDIR /app
COPY Cargo.toml ./
COPY src/ src/
COPY index.html Trunk.toml ./
RUN trunk build --release

FROM nginx:alpine
COPY --from=builder /app/dist /usr/share/nginx/html
COPY nginx.conf /etc/nginx/conf.d/default.conf
EXPOSE 8080
CMD ["nginx", "-g", "daemon off;"]
