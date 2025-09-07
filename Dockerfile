FROM rust:1.68-alpine

# Install dependencies
RUN apk add --no-cache openssl-dev musl-dev

# Create non-root user for security
RUN addgroup -g 1001 -S appgroup && \
    adduser -S -D -H -u 1001 -s /sbin/nologin -G appgroup appuser

WORKDIR /app

# Copy binary and set permissions before changing user
COPY target/x86_64-unknown-linux-musl/release/http-api http-api
RUN chmod +x http-api && \
    chown appuser:appgroup http-api && \
    chown appuser:appgroup /app && \
    chmod 755 /app

# Switch to non-root user
USER appuser

# Expose port (adjust to your app's port)
EXPOSE 8080

ENV RUST_LOG=info

# Add health check
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD wget --no-verbose --tries=1 --spider http://localhost:8080/api/v1/queue || exit 1

CMD ["./http-api"]
