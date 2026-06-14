FROM rust:latest

RUN cargo install cargo-watch

WORKDIR /app

CMD ["cargo", "watch", "-x", "run"]
