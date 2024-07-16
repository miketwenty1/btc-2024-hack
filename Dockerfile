# Use the official Rust image as the base image
FROM rust:latest

# Create a new directory for the application
WORKDIR /usr/src/btc-2024-hack

# Copy the source code
COPY . .

# Build the application in debug mode (default)
RUN cargo build

# Set the command to run the application
CMD ["cargo", "run", "--bin", "scale_bridge"]

# Expose the application port
EXPOSE 8080
