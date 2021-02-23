FROM messense/rust-musl-cross:x86_64-musl as cargo-build

WORKDIR /app

COPY Cargo.toml Cargo.toml

RUN mkdir src/

RUN echo "fn main() {println!(\"if you see this, the build broke\")}" > src/main.rs

RUN cargo build --release

RUN rm -f target/x86_64-musl/release/deps/vk_bot_repeat_rust*

COPY . .

RUN RUSTFLAGS=-Clinker=musl-gcc cargo build --release --target=x86_64-unknown-linux-musl

# -----------------------------------------------------

FROM alpine:3.12

COPY --from=cargo-build /app/target/x86_64-musl/release/deps/vk_bot_repeat_rust /app/bot

CMD ["/app/bot"]