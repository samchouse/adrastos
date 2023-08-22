FROM ubuntu:latest

WORKDIR /work
COPY target/release/adrastos .

CMD [ "./adrastos" ]
