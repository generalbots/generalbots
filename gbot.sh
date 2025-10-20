pkill postgres && rm -rf botserver-stack && clear && \
     RUST_LOG=trace,hyper_util=off cargo run 
