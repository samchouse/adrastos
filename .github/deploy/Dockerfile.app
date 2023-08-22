FROM ubuntu:latest

WORKDIR /work
COPY . .
RUN echo asd && ls -la && dfgdfgFSDKLFJSDJKLFLSDKFJ
COPY target/release/adrastos .

CMD [ "./adrastos" ]
