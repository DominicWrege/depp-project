    FROM rust:1.39
    RUN USER=root cargo new --bin deep-api
    WORKDIR /depp-api

    RUN apt-get update && apt-get install -y clang llvm-dev libclang-dev libmagic-dev

    COPY ./Cargo.lock ./Cargo.lock
    COPY ./Cargo.toml ./Cargo.toml

    RUN cargo build --release && rm src/*.rs

    COPY . .

    RUN cargo build --release

    FROM rust:1.39

    RUN apt-get update && apt-get install -y libmagic-dev

    COPY --from=0 /swt2-backend/target/release/swt2-backend .

    EXPOSE 8090

    CMD ["./deep-api"]
