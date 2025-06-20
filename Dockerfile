FROM rust:1.87 as builder

WORKDIR /app
COPY . .

RUN cargo build --release

FROM debian:bookworm-slim

WORKDIR /app
COPY --from=builder /app/target/release/hlmm /app/hlmm

ENTRYPOINT ["/app/hlmm"] 