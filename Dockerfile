FROM rust:1.68-alpine

RUN apk add openssl-dev musl-dev

COPY target/x86_64-unknown-linux-musl/release/http-api http-api

CMD ["./http-api"]
