# ============================================================
# Stage 1: Build the Rust binary on Alpine
# ============================================================
FROM rust:alpine AS builder

RUN apk add --no-cache musl-dev pkgconfig openssl-dev

WORKDIR /build

COPY Cargo.toml Cargo.lock ./
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo build --release 2>/dev/null || true
RUN rm -rf src

COPY config/ config/
COPY contents/ contents/
COPY src/ src/
RUN touch src/main.rs
RUN ls -laht
RUN cargo build --release

# ============================================================
# Stage 2: Minimal runtime image
# ============================================================
FROM alpine:3.21

RUN apk add --no-cache ca-certificates tzdata

ARG UID=1000
ARG GID=1000

RUN addgroup -S dx5 -g ${GID} && adduser -S dx5 -G dx5 -u ${UID}

WORKDIR /app

COPY --from=builder /build/target/release/dx5 /app/dx5

COPY config/ config/
COPY contents/ contents/
COPY templates/ templates/
COPY static/ static/
COPY assets/ assets/
COPY Rocket.toml .

RUN chown -R dx5:dx5 /app

USER dx5

EXPOSE 8000

ENV ROCKET_ADDRESS=0.0.0.0
ENV ROCKET_PORT=8000

CMD ["/app/dx5"]
