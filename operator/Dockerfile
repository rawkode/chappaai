FROM rust AS build

WORKDIR /src
COPY . /src

RUN cargo build --release --bin crdgen
RUN cargo build --release --bin chappaai

FROM rust

COPY --from=build /src/target/release/crdgen /usr/bin/crdgen
COPY --from=build /src/target/release/chappaai /usr/bin/chappaai

ENTRYPOINT ["/usr/bin/chappaai"]
