FROM ubuntu:latest

RUN apt-get update && apt-get install -y ca-certificates && apt-get clean

WORKDIR /work
COPY target/release/adrastos .

CMD [ "./adrastos" ]
