FROM rust:1.88-bookworm AS chef
# Default build profile is dev
ARG BUILD_PROFILE=dev
RUN apt update && apt install -y ca-certificates
RUN cargo install cargo-chef --locked

FROM chef AS planner
WORKDIR /app
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder
WORKDIR /app
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --profile ${BUILD_PROFILE} --locked --recipe-path recipe.json
COPY . .
RUN cargo install --profile ${BUILD_PROFILE} --locked --path .

FROM debian:bookworm-slim AS runtime
RUN apt update && apt install -y subnetcalc
COPY --from=builder /usr/local/cargo/bin/netmath-back-end /usr/bin/
ENTRYPOINT [ "netmath-back-end" ]
