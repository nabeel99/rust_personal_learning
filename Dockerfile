FROM lukemathwalker/cargo-chef:latest-rust-1.66.0 AS chef
WORKDIR /app


FROM chef AS planner
COPY . .
# Compute a lock-like file for our project
RUN cargo chef prepare --recipe-path recipe.json


#import a basic image

FROM chef AS builder
COPY --from=planner /app/recipe.json recipe.json
# Build our project dependencies, not our application , cache layer for dependencies
RUN cargo chef cook --release --recipe-path recipe.json
# Up to this point, if our dependency tree stays the same,
# all layers should be cached.
COPY . .
#equivalent to mkdikr app-> cd app
#ARG Syntax, <variablename> = <default value>, arg value can be passed in at build time via 
#--build-arg if no value is passed default value is used
#ENV IS Persisted in the final image, ARG is not
# ARG binary_name="rust_personal_learning"
ENV SQLX_OFFLINE=true
#WORKDIR /app
#NO NEED NOW AS CHEF image has it - RUN apt update && apt install lld clang -y


RUN cargo build --release --bin rust_personal_learning
#after the binary is built we can use the multi stage build pattern to ditch it out as a intermediate container
#and use the final binary
#FROM rust:1.66.0-slim as runtime
#use a even more slimmed down version of the o.s
# We do not need the Rust toolchain to run the binary!
FROM debian:bullseye-slim AS runtime
WORKDIR /app
# Install OpenSSL - it is dynamically linked by some of our dependencies
# Install ca-certificates - it is needed to verify TLS certificates
# when establishing HTTPS connections
RUN apt-get update -y \
    && apt-get install -y --no-install-recommends openssl ca-certificates \
    #clean up
    && apt-get autoremove -y \
    && apt-get clean -y \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/rust_personal_learning rust_personal_learning
COPY configuration configuration

ENV APP_ENVIRONMENT="production"

ENTRYPOINT ["./rust_personal_learning"]
#build context : what files docker can see on your host system and interact with.