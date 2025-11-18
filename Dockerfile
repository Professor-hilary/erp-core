# Stage 1: build
FROM rust:1.73 AS builder
WORKDIR /usr/src/chiefalry

# Cache deps
COPY Cargo.toml Cargo.lock ./
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo build --release || true

# Copy actual source
COPY . .
RUN cargo build --release -Z unstable-options --out-dir /usr/local/bin

# Stage 2: runtime (debian-slim)
FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates libssl3 && rm -rf /var/lib/apt/lists/*
COPY --from=builder /usr/local/bin/chiefalry /usr/local/bin/chiefalry
# Add a small migration binary if seperate
COPY --from=builder /usr/local/bin/migrator /usr/local/bin/migrator

ENV RUST_LOG=info
EXPOSE 8000
ENTRYPOINT [ "/usr/local/bin/chiefalry" ]
CMD [ "serve" ]