FROM rust:1.68-slim-bullseye AS builder
WORKDIR app
RUN apt update && apt install git --no-install-recommends --yes
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
