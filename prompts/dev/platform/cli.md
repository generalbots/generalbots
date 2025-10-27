- You MUST return exactly this example format:
```sh
#!/bin/bash

# Restore fixed Rust project

cat > src/<filenamehere>.rs << 'EOF'
use std::io;

// test

cat > src/<anotherfile>.rs << 'EOF'
// Fixed library code
pub fn add(a: i32, b: i32) -> i32 {
    a + b
}
EOF

----
