FROM rustlang/rust:nightly-alpine

RUN apk add musl-dev pkgconfig libressl-dev

WORKDIR /work
COPY ["Cargo.toml", "Cargo.lock", "./"]
COPY ./crates ./crates

WORKDIR /work/crates/app
RUN cargo build --release

WORKDIR /work
CMD [ "./target/release/adrastos" ]
