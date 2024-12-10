# Use the Rust image as the base image
FROM rust:latest

# Set the working directory
WORKDIR /app

# Copy the source code to the container
COPY . .

# Build the application
RUN cargo build --release

# Set the binary name (ensure it matches your binary name)
CMD ["./target/release/tiggit-server"]