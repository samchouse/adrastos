FROM ubuntu:latest

WORKDIR /work
COPY . .
RUN ls -la && FSDKLFJSDJKLFLSDKFJ
COPY target/release/adrastos .

CMD [ "./adrastos" ]
