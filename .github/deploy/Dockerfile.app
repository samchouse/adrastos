FROM ubuntu:latest

WORKDIR /work
COPY . .
RUN ls -la && dfgdfgFSDKLFJSDJKLFLSDKFJ
COPY target/release/adrastos .

CMD [ "./adrastos" ]
