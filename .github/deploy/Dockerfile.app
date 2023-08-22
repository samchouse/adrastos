FROM ubuntu:latest

RUN ls -la && exit 1
COPY target/release/adrastos .

CMD [ "./adrastos" ]
