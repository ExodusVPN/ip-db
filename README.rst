IANA-IP
=========



.. code:: bash

    cargo build --bin iana --release

    target/debug/iana sync
    target/debug/iana parse

    target/debug/iana lookup "23.18.0.0"
    target/debug/iana lookup "2001:258:4000::"
