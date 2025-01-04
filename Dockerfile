FROM rust:1.75-slim as builder

WORKDIR /usr/src/ghif
COPY . .

RUN cargo build --release

FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*

# Create a non-root user
RUN useradd -m -u 1000 ghif

# Create and set permissions for the output directory
RUN mkdir -p /issues && chown -R ghif:ghif /issues

USER ghif

COPY --from=builder /usr/src/ghif/target/release/ghif /usr/local/bin/ghif

ENTRYPOINT ["ghif"]