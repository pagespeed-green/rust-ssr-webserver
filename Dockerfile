FROM rust:1.41 as server_build
LABEL maintainer="alex@pagespeed.green"

COPY . .
RUN cargo build --release

FROM rust:slim
COPY --from=server_build target/release/ssr_webserver /bin/
COPY --from=server_build cert cert
COPY /static static

CMD ssr_webserver
