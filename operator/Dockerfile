FROM rust AS build

WORKDIR /src
COPY . /src

RUN cargo build --release --bin crdgen
RUN cargo build --release --bin controller

FROM rust

COPY --from=build /src/release/crdgen /usr/bin/crdgen
COPY --from=build /src/release/controller /usr/bin/controller

ENTRYPOINT ["/usr/bin/controller"]
