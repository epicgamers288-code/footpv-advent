FROM rust:1.100-bookworm AS build
WORKDIR /project
COPY server/ server/
COPY client/ client/
WORKDIR /project/server
RUN cargo build --locked --release -j 2
FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=build /project/server/target/release/footpv-advent /usr/local/bin/advent
USER 65534:65534
ENTRYPOINT ["/usr/local/bin/advent"]
