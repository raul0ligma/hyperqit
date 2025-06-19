FROM rust:1.76 as builder

WORKDIR /app
COPY . .

RUN cargo build --release

FROM debian:bookworm-slim

WORKDIR /app
COPY --from=builder /app/target/release/hlmm /app/hlmm

# If you need environment variables, mount or pass them at runtime
ENTRYPOINT ["/app/hlmm"] 