FROM rust:1.76.0-bookworm as builder

WORKDIR /usr/src/ord

COPY . .

RUN cargo build --bin ord --release

FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y libssl3

COPY --from=builder /usr/src/ord/target/release/ord /usr/local/bin
RUN apt-get update && apt-get install -y openssl

ENV RUST_BACKTRACE=1
ENV RUST_LOG=info

ENTRYPOINT ["/usr/local/bin/ord"]
