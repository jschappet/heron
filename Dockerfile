FROM rust:1.92.0 AS builder
WORKDIR /usr/src/app

# Install system deps early (cached)
RUN apt-get update && apt-get install -y \
    libsqlite3-dev \
    && rm -rf /var/lib/apt/lists/*

# Copy only Cargo manifests
COPY Cargo.toml Cargo.lock build.rs ./
COPY migrations ./migrations
COPY diesel.toml ./


# Create a dummy main to compile dependencies
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo build --release
RUN rm -rf src

# Now copy real source
COPY src ./src
COPY templates ./templates

# Build actual binary
RUN cargo build --release


