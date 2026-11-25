FROM rust:1.100-bookworm AS build
WORKDIR /src
COPY . .
ARG FEATURES=""
RUN cargo build --locked --release --features "$FEATURES"
FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=build /src/target/release/footpv-advent /usr/local/bin/advent
USER 65534:65534
ENTRYPOINT ["/usr/local/bin/advent"]
