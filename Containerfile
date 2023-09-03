FROM rust:1.72.0-alpine as build

WORKDIR /src

RUN apk add --no-cache musl-dev

COPY . /src/

RUN cargo build --release

FROM scratch

COPY --from=build /src/target/release/peek-reverse-proxy /peek-reverse-proxy

CMD ["/peek-reverse-proxy"]