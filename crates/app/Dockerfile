# Step 1
FROM rustlang/rust:nightly-alpine AS chef

RUN apk add musl-dev
RUN cargo install cargo-chef

WORKDIR /work

# Step 2
FROM chef AS planner

COPY ["Cargo.toml", "Cargo.lock", "./"]
COPY ./crates ./crates

RUN cargo chef prepare --recipe-path recipe.json

# Step 3
FROM chef AS builder

COPY --from=planner /work/recipe.json recipe.json
RUN apk add libressl-dev
RUN cargo chef cook --release --recipe-path recipe.json

COPY ["Cargo.toml", "Cargo.lock", "./"]
COPY ./crates ./crates

WORKDIR /work/crates/app
RUN cargo build --release

# Step 4
FROM alpine:latest AS runtime

WORKDIR /work
COPY --from=builder /work/target/release/adrastos .

CMD [ "./adrastos" ]
