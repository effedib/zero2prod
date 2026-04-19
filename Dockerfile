FROM rust:1.94.1

WORKDIR /app

RUN apt update && apt install clang lld mold -y

COPY . .

RUN cargo build --release

ENTRYPOINT [ "./target/release/zero2prod" ]

