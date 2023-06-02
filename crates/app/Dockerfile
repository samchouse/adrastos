FROM rustlang/rust:nightly-alpine as builder

RUN apk add musl-dev pkgconfig libressl-dev

WORKDIR /work
COPY ["Cargo.toml", "Cargo.lock", "./"]
COPY ./crates ./crates

WORKDIR /work/crates/app
RUN cargo build --release

FROM alpine:latest

WORKDIR /work
COPY --from=builder /work/target/release/adrastos .

CMD [ "./adrastos" ]
