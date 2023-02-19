FROM rust:1.67.1-bullseye AS builder
RUN apt-get update && apt-get install -y pkg-config libssl-dev
COPY src /build/src
COPY Cargo* /build
WORKDIR /build
RUN cargo install --locked --path .

FROM debian:bullseye-slim
RUN apt-get update && apt-get install -y libssl1.1 libcap2-bin && rm -rf /var/lib/apt/lists/*
COPY --from=builder /usr/local/cargo/bin/home-monitor-rs /usr/local/bin/home-monitor-rs
RUN setcap cap_net_raw=eip /usr/local/bin/home-monitor-rs
RUN mkdir -p /etc/home-monitor-rs/
ENTRYPOINT [ "/usr/local/bin/home-monitor-rs" ]
