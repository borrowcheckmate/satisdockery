FROM rust:1-trixie AS builder
WORKDIR /work
COPY Cargo.toml Cargo.lock ./
COPY src ./src
COPY graphql ./graphql
RUN cargo build --release --locked

FROM debian:trixie
COPY --from=builder /work/target/release/satisdockery /usr/bin/
COPY docker/init.sh /usr/bin/init.sh

ARG TARGETARCH

RUN groupadd -g 1000 satisdockery && useradd satisdockery -u 1000 -g 1000 -m -s /bin/bash
WORKDIR /home/satisdockery
RUN if [ "$TARGETARCH" = "arm64" ] || [ "$TARGETARCH" = "aarch64" ]; then \
      apt update && apt install wget -y && \
      wget https://github.com/Pi-Apps-Coders/box64-debs/raw/2620c04f7b84daddaac89ab5a010697fba8709e7/debian/box64-generic-arm_0.4.3+20260606T083319.e694f2c-1_arm64.deb && \
      echo "208eac2354a7c161711c5b0195e704190518d718c124e8b81007dbfdbfcf5da9  box64-generic-arm_0.4.3+20260606T083319.e694f2c-1_arm64.deb" | sha256sum --check && \
      dpkg -i box64-generic-arm_0.4.3+20260606T083319.e694f2c-1_arm64.deb && \
      rm box64-generic-arm_0.4.3+20260606T083319.e694f2c-1_arm64.deb && \
      apt remove wget -y && \
      apt clean; \
    else \
      echo "not aarch64: skipping box64 install"; \
    fi

USER satisdockery
ENV SATISDOCKERY_PATH=/home/satisdockery/satisfactory
ENV UPDATE="true"

# ENTRYPOINT [ "init.sh" ]
