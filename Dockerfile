# Build xezim in a container and get a ready-to-run binary out of it.
#
#   Image with xezim on PATH:
#     docker build -t xezim .
#     docker run --rm -v "$PWD:/work" xezim tb.sv
#
#   Just the binary, written to ./dist on the host:
#     docker build --target export --output type=local,dest=dist .
#
# The binary links against glibc 2.36 (Debian bookworm): it runs on any
# Linux x86-64/arm64 host whose glibc is 2.36 or newer.
#
# Build arguments:
#   PROFILE   cargo profile: release (default) or release-lto
#   FEATURES  extra cargo features, e.g. jit

ARG RUST_VERSION=1.92

# The rust image already has what the build needs besides cargo: git (the
# pinned xezim-core dependency), a C compiler (the VPI/DPI shims) and
# autoconf/automake/libtool (the vendored libffi).
FROM rust:${RUST_VERSION}-bookworm AS build
WORKDIR /src
COPY . .
ARG PROFILE=release
ARG FEATURES=
# XEZIM_UVM_DIR: build.rs otherwise clones the UVM library for the test
# suite, which a binary build does not need.
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/usr/local/cargo/git \
    --mount=type=cache,target=/src/target \
    XEZIM_UVM_DIR=/nonexistent \
    cargo build --profile "$PROFILE" --bin xezim \
        ${FEATURES:+--features "$FEATURES"} \
 && mkdir -p /out \
 && cp "target/$PROFILE/xezim" /out/xezim \
 && strip /out/xezim

# `--target export --output type=local,dest=dist`: the binary plus the DPI/VPI
# headers, and nothing else.
FROM scratch AS export
COPY --from=build /out/xezim /xezim
COPY include /include

# Runtime image: the binary and the svdpi.h / vpi_user.h headers. A DPI-C or
# VPI library is a shared object built against those headers (on the host,
# or `cc -shared -fPIC -I/usr/local/include/xezim ...` in an image with a
# compiler) and passed with --dpi-lib / --vpi.
FROM debian:bookworm-slim
COPY --from=build /out/xezim /usr/local/bin/xezim
COPY include /usr/local/include/xezim
WORKDIR /work
ENTRYPOINT ["xezim"]
