clear && \
     cargo build && \
     sudo RUST_BACKTRACE=1 ./target/debug/botserver install tables
