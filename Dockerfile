FROM rust:alpine3.22 AS build

WORKDIR /app

RUN apk add --no-cache musl-dev libseccomp-dev libseccomp-static

COPY src ./src
COPY Cargo.toml .
RUN cargo build --release

FROM scratch
COPY --from=build /app/target/release/mymoulette /mymoulette
ENTRYPOINT ["/mymoulette"]