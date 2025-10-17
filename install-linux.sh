sudo apt install pkg-config -y &&
cargo build --release &&
sudo cp target/release/hf-cli /usr/bin