FROM rust:1.99-trixie AS builder
WORKDIR /work
COPY Cargo.toml Cargo.lock ./
# Compile the dependencies in their own layer so source changes do not rebuild them.
RUN mkdir src && echo 'fn main() {}' > src/main.rs && touch src/lib.rs \
    && cargo build --release --locked
COPY src ./src
COPY graphql ./graphql
RUN touch src/main.rs src/lib.rs && cargo build --release --locked

FROM debian:trixie-slim
ARG TARGETARCH
ARG BOX64_DEB=box64-generic-arm_0.4.3+20260606T083319.e694f2c-1_arm64.deb
ARG BOX64_SHA256=208eac2354a7c161711c5b0195e704190518d718c124e8b81007dbfdbfcf5da9

# The Satisfactory server only exists for x86_64, so arm64 runs it through box64.
RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates \
    && if [ "$TARGETARCH" = "arm64" ]; then \
        apt-get install -y --no-install-recommends wget \
        && wget -q -O /tmp/box64.deb "https://github.com/Pi-Apps-Coders/box64-debs/raw/2620c04f7b84daddaac89ab5a010697fba8709e7/debian/${BOX64_DEB}" \
        && echo "${BOX64_SHA256}  /tmp/box64.deb" | sha256sum --check \
        && apt-get install -y /tmp/box64.deb \
        && rm /tmp/box64.deb \
        && apt-get purge -y --auto-remove wget; \
    fi \
    && rm -rf /var/lib/apt/lists/*

RUN groupadd -g 1000 satisdockery \
    && useradd satisdockery -u 1000 -g 1000 -m -s /bin/bash \
    && install -d -o satisdockery -g satisdockery /home/satisdockery/satisfactory

COPY --from=builder /work/target/release/satisdockery /usr/bin/satisdockery

USER satisdockery
WORKDIR /home/satisdockery
ENV SATISDOCKERY_PATH=/home/satisdockery/satisfactory
EXPOSE 7777/udp 8888/tcp
ENTRYPOINT ["satisdockery"]
