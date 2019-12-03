FROM rust:1.39 as builder
RUN USER=root cargo new deep-api --lib
WORKDIR /deep-api

COPY ./Cargo.lock ./Cargo.lock
COPY ./Cargo.toml ./Cargo.toml

RUN cargo build --release && rm src/*.rs

COPY . .

RUN cargo build --release

FROM rust:1.39

RUN apt-get update &&\
    apt-get install -y \
        less \
        locales \
        ca-certificates \
        libicu63 \
        libssl1.1 \
        libc6 \
        libgcc1 \
        libgssapi-krb5-2 \
        liblttng-ust0 \
        libstdc++6 \
        zlib1g \
        curl &&\
    curl -L https://github.com/PowerShell/PowerShell/releases/download/v7.0.0-preview.4/powershell-7.0.0-preview.4-linux-x64.tar.gz -o /tmp/powershell.tar.gz &&\
    mkdir -p /opt/microsoft/powershell/7-preview &&\
    tar zxf /tmp/powershell.tar.gz -C /opt/microsoft/powershell/7-preview &&\
    chmod +x /opt/microsoft/powershell/7-preview/pwsh &&\
    ln -s /opt/microsoft/powershell/7-preview/pwsh /usr/bin/pwsh &&\
    apt-get install -y --no-install-recommends wine python3

COPY --from=builder /deep-api/target/release/deep-api .
EXPOSE 8088

CMD ["./deep-api","--config", "/media/examples/assignments.toml"]
