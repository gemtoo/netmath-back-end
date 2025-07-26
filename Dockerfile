FROM rust:1.88-alpine AS chef
# Default build profile is dev
ARG BUILD_PROFILE=dev
RUN apk add --no-cache ca-certificates musl-dev
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

FROM alpine:latest AS runtime
COPY --from=builder /usr/local/cargo/bin/netmath-back-end /usr/bin/
ENTRYPOINT [ "netmath-back-end" ]
