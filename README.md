
# ryzc
```bash
sudo apt-get install gcc qemu -y
curl https://sh.rustup.rs -sSf | sh -s -- -y
export PATH="${PATH}:$HOME/.cargo/bin"
rustup default nightly
rustup component add rust-src
rustup component add llvm-tools-preview
cargo install cargo-xbuild
cargo install bootimage
bootimage run
```

## Running

### qemu

```bash
bootimage run
qemu-system-x86_64 -drive format=raw,file=./target/x86_64-ryzc/debug/bootimage-ryzc.bin -hdb ./test.img
```
