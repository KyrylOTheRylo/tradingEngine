FROM rust:1.66


WORKDIR /usr/src/myapp
COPY . .

RUN cargo build --release

CMD cargo run