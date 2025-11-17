FROM rust:alpine3.22 AS build

WORKDIR /app
COPY src ./src
COPY Cargo.toml .

RUN cargo build --release

FROM scratch
COPY --from=build /app/target/release/mymoulette /mymoulette
ENTRYPOINT ["/mymoulette"]