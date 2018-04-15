IANA-IP
=========



.. code:: bash
    
    cargo build
    cargo build --bin sync --features="sync"
    cargo build --bin parse --features="parse"
    cargo build --bin lookup

    target/debug/sync
    target/debug/parse # Parse RIR DB File and codegen.

    target/debug/lookup "23.18.0.0"
    