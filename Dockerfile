FROM rust:1-bookworm as builder

WORKDIR /usr/src/app
COPY . .
# Will build and cache the binary and dependent crates in release mode
RUN --mount=type=cache,target=/usr/local/cargo,from=rust:latest,source=/usr/local/cargo \
    --mount=type=cache,target=target \
    cargo build --release && mv ./target/release/rust_scribbl ./rust_scribbl

# Run yarn build inside the client directory
WORKDIR /usr/src/app/client
RUN yarn build

# Runtime image
FROM debian:bookworm-slim

# Run as "app" user
RUN useradd -ms /bin/bash app

USER app
WORKDIR /app

# Get compiled binaries from builder's cargo install directory
COPY --from=builder /usr/src/app/rust_scribbl /app/rust_scribbl
COPY /dist /app/dist
COPY /templates /app/templates
COPY /words.txt /app/words.txt

# Run the app
CMD ./rust_scribbl