FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    wget \
    && rm -rf /var/lib/apt/lists/*

# Create non-root user for security
RUN groupadd -g 1001 appgroup && \
    useradd -r -u 1001 -g appgroup -s /usr/sbin/nologin appuser

WORKDIR /app/data

# Create a data directory for volume mount and set permissions
RUN mkdir -p /app/data && \
    chown appuser:appgroup /app/data && \
    chmod 755 /app/data

# Declare /app/data as a volume (optional, for documentation and best practices)
VOLUME ["/app/data"]

# Copy binary and set permissions before changing user
COPY target/x86_64-unknown-linux-gnu/release/http-api /app/http-api
RUN chmod +x /app/http-api && \
    chown appuser:appgroup /app/http-api && \
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

CMD ["/app/http-api"]
