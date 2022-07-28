FROM rust:1.62-slim-bullseye AS chef
WORKDIR app
RUN cargo install cargo-chef --locked

FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder
COPY --from=planner /app/recipe.json recipe.json
# Build dependencies - this is the caching Docker layer!
RUN ls -la
RUN cargo chef cook --recipe-path recipe.json
# Build application
COPY . .
RUN cargo build

FROM debian:bullseye-slim AS runtime
WORKDIR app
COPY --from=builder /app/target/debug/lemmy_bb .
COPY assets assets
COPY templates templates
ENTRYPOINT ["/app/lemmy_bb"]
