####################################################################################################
## Builder
####################################################################################################
FROM rust:latest AS builder

RUN rustup target add x86_64-unknown-linux-musl
RUN apt update && apt-get -y install build-essential && apt install -y libpq-dev
RUN apt-get install -y libssl-dev
RUN update-ca-certificates

# Create appuser
ENV USER=krusty
ENV UID=10001

RUN adduser \
    --disabled-password \
    --gecos "" \
    --home "/nonexistent" \
    --shell "/sbin/nologin" \
    --no-create-home \
    --uid "${UID}" \
    "${USER}"


WORKDIR /krusty

COPY ./ .

RUN rustup update
RUN rustup component add clippy
RUN rustup install nightly

RUN cargo --version --verbose
RUN rustc --version
RUN cargo clippy --version

RUN cargo check --release
RUN cargo clippy -- -D warnings
RUN cargo test --release
RUN cargo build --release

####################################################################################################
## Final image
####################################################################################################
FROM ubuntu:latest

RUN apt update && apt-get -y install build-essential && apt install -y libpq-dev
RUN apt-get install -y libssl-dev

# Import from builder.
COPY --from=builder /etc/passwd /etc/passwd
COPY --from=builder /etc/group /etc/group

WORKDIR /krusty

# Copy our build
COPY --from=builder /krusty/target/release/krusty ./

# Use an unprivileged user.
USER krusty:krusty

CMD ["/krusty/krusty"]
