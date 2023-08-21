FROM alpine:latest AS builder

RUN apk add curl gcc musl-dev libressl-dev protobuf-dev

RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- --default-toolchain none -y
ENV PATH="/root/.cargo/bin:${PATH}"

RUN rustup toolchain install nightly --profile minimal --no-self-update
RUN rustup default nightly

ENV CARGO_INCREMENTAL=0

WORKDIR /work

COPY ["Cargo.toml", "Cargo.lock", "./"]
COPY ./crates ./crates

WORKDIR /work/crates/app
RUN cargo build --release

# Step 4
FROM alpine:latest AS runtime

WORKDIR /work
COPY --from=builder /work/target/release/adrastos .

CMD [ "./adrastos" ]
