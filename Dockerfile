FROM lukemathwalker/cargo-chef:latest-rust-1-slim-trixie AS chef
WORKDIR /app
RUN apt update && apt install clang lld mold -y


FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json


FROM chef AS builder
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json
COPY . .
ENV SQLX_OFFLINE=true
RUN cargo build --release --bin zero2prod


FROM debian:trixie-slim AS runtime
WORKDIR /app

# ####################################
# if use rust:1.94.1-slim as runtime,
# the entire RUN block is not needed
RUN apt-get update -y \
  && apt-get install -y --no-install-recommends openssl ca-certificates \
  # clean up 
  && apt-get autoremove -y \
  && apt-get clean -y \
  && rm -rf /var/lib/apt/lists/*
# ####################################

COPY --from=builder /app/target/release/zero2prod zero2prod
COPY configuration configuration
ENV APP_ENVIRONMENT=production
ENTRYPOINT [ "/app/zero2prod" ]
