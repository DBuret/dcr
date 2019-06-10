FROM devalias/upx:devel AS upx

FROM ubuntu:xenial as builder
# add some tools
RUN apt-get -y update && apt-get -y install curl g++ libssl-dev pkg-config musl-tools ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=upx /usr/bin/upx /usr/bin/upx

# install rust
# ENV RUST_VERSION 1.35
ENV RUST_VERSION stable
RUN curl https://sh.rustup.rs -sSf | sh -s -- -y --default-toolchain ${RUST_VERSION}

#setup rust working env
ENV PATH $PATH:/root/.cargo/bin
RUN rustup target add x86_64-unknown-linux-musl
RUN mkdir source && mkdir .cargo && echo "[target.x86_64-unknown-linux-musl]\n" > .cargo/config
ENV SSL_VER 1.0.2o
ENV CC musl-gcc
ENV PREFIX /usr/local
ENV PATH /usr/local/bin:$PATH
ENV PKG_CONFIG_PATH /usr/local/lib/pkgconfig

#install and config openssl
RUN curl -sL http://www.openssl.org/source/openssl-$SSL_VER.tar.gz | tar xz \
    &&  cd openssl-$SSL_VER \
    &&  ./Configure no-shared --prefix=$PREFIX --openssldir=$PREFIX/ssl no-zlib linux-x86_64 -fPIC \
    &&  make -j$(nproc) && make install && cd .. && rm -rf openssl-$SSL_VER
ENV SSL_CERT_FILE /etc/ssl/certs/ca-certificates.crt
ENV SSL_CERT_DIR /etc/ssl/certs
ENV OPENSSL_LIB_DIR $PREFIX/lib
ENV OPENSSL_INCLUDE_DIR $PREFIX/include
ENV OPENSSL_DIR $PREFIX
ENV OPENSSL_STATIC 1

# compile 
ENV PKG_CONFIG_ALLOW_CROSS 1

# could move to another stage here...

WORKDIR /source
COPY ./ ./
ENV PKG_CONFIG_ALLOW_CROSS=1
RUN cargo build --target x86_64-unknown-linux-musl --release

# !!!! put your binary name here
RUN  /usr/bin/upx target/x86_64-unknown-linux-musl/release/dcr

#display result of build stage
RUN ls -al target/x86_64-unknown-linux-musl/release

## move to final image creation
FROM scratch
LABEL version="0.1"
LABEL link="https://github.com/DBuret/dcr"
LABEL description="Debug Containers with Rust - micro HTTP service to help understanding container orchestrators environment"
COPY --from=builder /etc/ssl/certs/ca-certificates.crt /etc/ssl/certs/ca-certificates.crt
COPY --from=builder /source/target/x86_64-unknown-linux-musl/release/dcr /
COPY --from=builder /source/static  /static

ENV SSL_CERT_FILE=/etc/ssl/certs/ca-certificates.crt
ENV SSL_CERT_DIR=/etc/ssl/certs

CMD ["/dcr"]
