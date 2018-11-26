IANA-IP
=========

:Date: 2018/11/26


.. contents:: 


Build & Run
------------------

.. code:: bash
    
    cargo build
    cargo build --bin sync --features="sync"
    cargo build --bin parse --features="parse"
    cargo build --bin lookup

    ./target/debug/sync
    ./target/debug/parse # Parse RIR DB File and codegen.

    ./target/debug/lookup "23.18.0.0"


已知问题
-----------

*2018/11/26*  因为国家代码 `AP` 不在国家代码表里面，尚且不知道 `AP` 代表哪个国家，因此跟这个国家代号相关的一些数据记录被丢弃::

    data/delegated-apnic-latest:apnic|AP|asn|55767|1|20100916|allocated
    data/delegated-apnic-latest:apnic|AP|asn|55784|1|20100923|allocated
    data/delegated-apnic-latest:apnic|AP|ipv4|182.50.184.0|2048|20100302|allocated
    data/delegated-apnic-latest:apnic|AP|ipv6|2402:d00::|32|20110926|allocated
    data/delegated-apnic-extended-latest:apnic|AP|ipv4|182.50.184.0|2048|20100302|allocated|A9177358
    data/delegated-apnic-extended-latest:apnic|AP|ipv6|2402:d00::|32|20110926|allocated|A9177358
    data/delegated-apnic-extended-latest:apnic|AP|asn|55767|1|20100916|allocated|A9177358
    data/delegated-apnic-extended-latest:apnic|AP|asn|55784|1|20100923|allocated|A9177358

