FROM rust:1-bookworm as builder

# Install Node.js and Yarn
RUN curl -fsSL https://deb.nodesource.com/setup_18.x | bash - && \
    apt-get install -y nodejs && \
    npm install -g yarn

WORKDIR /usr/src/app
COPY . .
# Will build and cache the binary and dependent crates in release mode
RUN --mount=type=cache,target=/usr/local/cargo,from=rust:latest,source=/usr/local/cargo \
    --mount=type=cache,target=target \
    cargo build --release && mv ./target/release/skribbl ./skribbl

# Run yarn build inside the client directory
WORKDIR /usr/src/app/client
RUN yarn install && yarn build

# Runtime image
FROM debian:bookworm-slim

# Run as "app" user
RUN useradd -ms /bin/bash app

USER app
WORKDIR /app

# Get compiled binaries from builder's cargo install directory
COPY --from=builder /usr/src/app/skribbl /app/skribbl
COPY --from=builder /usr/src/app/dist /app/dist
COPY --from=builder /usr/src/app/templates /app/templates
COPY --from=builder /usr/src/app/Rocket.toml /app/Rocket.toml

# Run the app
CMD ./skribbl