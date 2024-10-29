#Use the official Rust image as a base
FROM rust:latest AS builder

# Set the working directory inside the container
WORKDIR /usr/src/p2p_chat

# Copy your Rust source code into the container
COPY . .

# Build the application in release mode
RUN cargo build --release

# Use nicolaka/netshoot as the base image
FROM nicolaka/netshoot

# Set up a working directory (optional)
WORKDIR /usr/src/p2p_chat

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    libc6 

# Copy your Rust binary from a local directory into the container
COPY --from=builder /usr/src/p2p_chat/target/release/p2p_chat /usr/local/bin/p2p_chat

# Set default command to start a shell
CMD ["bash"]
