FROM rust:1.76-buster as builder
WORKDIR /usr/src/app

RUN apt-get update && apt-get install -y protobuf-compiler libprotobuf-dev

COPY ./Cargo.toml ./Cargo.toml
COPY ./Cargo.lock ./Cargo.lock
COPY ./src ./src
COPY ./build.rs ./build.rs
COPY ./proto ./proto

RUN cargo build --release

FROM debian:buster-slim as runner

COPY --from=builder /usr/src/app/target/release/api-gateway /usr/local/bin/api-gateway
COPY ./Rocket.toml ./Rocket.toml
EXPOSE 8000
ENV AUTH_GRPC_URI=http://auth.incendio.svc.cluster.local:50051

CMD ["api-gateway"]
