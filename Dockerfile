# Use the official Rust image as the base image
FROM rust:latest

# Create a new directory for the application
WORKDIR /usr/src/btc-2024-hack

# Copy the source code
COPY . .

# Build the application
RUN cargo build --release

# Set the command to run the application
CMD ["./target/release/scale_bridge"]

# Expose the application port
EXPOSE 8080