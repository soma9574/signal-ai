# Dockerfile for Signal AI Chatbot on Railway

# Use a Rust base image for building the application
FROM rust:1.70 as builder

# Set working directory
WORKDIR /usr/src/signal-ai

# Copy the project files
COPY . .

# Build the Rust application
RUN cd backend && cargo build --release

# Use a lightweight base image for the final container
FROM debian:bullseye-slim

# Install necessary dependencies for signal-cli and runtime
RUN apt-get update && apt-get install -y \
    openjdk-17-jre \
    wget \
    tar \
    && rm -rf /var/lib/apt/lists/*

# Install signal-cli
ENV SIGNAL_CLI_VERSION=0.13.17
RUN wget https://github.com/AsamK/signal-cli/releases/download/v${SIGNAL_CLI_VERSION}/signal-cli-${SIGNAL_CLI_VERSION}.tar.gz \
    && tar xf signal-cli-${SIGNAL_CLI_VERSION}.tar.gz -C /opt \
    && ln -sf /opt/signal-cli-${SIGNAL_CLI_VERSION}/bin/signal-cli /usr/local/bin/signal-cli \
    && rm signal-cli-${SIGNAL_CLI_VERSION}.tar.gz

# Copy the built binary from the builder stage
COPY --from=builder /usr/src/signal-ai/backend/target/release/backend /usr/local/bin/backend

# Set environment variables (will be overridden by Railway if set in dashboard)
ENV ANTHROPIC_API_KEY=""
ENV SIGNAL_PHONE_NUMBER=""
ENV DATABASE_URL=""

# Expose the port for the application
EXPOSE 3000

# Run the application
CMD ["backend"] 