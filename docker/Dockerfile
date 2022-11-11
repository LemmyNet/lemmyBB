FROM rust:1.65-slim-bullseye AS chef
WORKDIR app
RUN cargo install cargo-chef --locked

FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder
COPY --from=planner /app/recipe.json recipe.json
# Build dependencies - this is the caching Docker layer!
RUN cargo chef cook --release --recipe-path recipe.json
RUN apt update && apt install git --no-install-recommends --yes
# Build application
COPY . .
RUN LEMMYBB_VERSION=$(git describe --tag --always) cargo build --release

FROM debian:bullseye-slim AS runtime
WORKDIR app
COPY --from=builder /app/target/release/lemmy_bb .
COPY assets assets
COPY templates templates
COPY lemmy-translations lemmy-translations
COPY lemmybb-translations lemmybb-translations
ENTRYPOINT ["/app/lemmy_bb"]
