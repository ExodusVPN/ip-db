extern crate iana;

use iana::lookup;

#[allow(unused_imports)]
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};


fn main (){
    // https://developers.google.com/speed/public-dns/docs/using?csw=1
    println!("{:?}", lookup(&"8.8.8.8".parse().unwrap()) );
    println!("{:?}", lookup(&"8.8.4.4".parse().unwrap()) );
    println!("{:?}", lookup(&"2001:4860:4860::8888".parse().unwrap()) );
    println!("{:?}", lookup(&"2001:4860:4860::8844".parse().unwrap()) );

    // https://www.opendns.com/about/innovations/ipv6/
    println!("{:?}", lookup(&"2620:0:ccc::2".parse().unwrap()) );
    println!("{:?}", lookup(&"2620:0:ccd::2".parse().unwrap()) );
    
    // JP
    // 2001:218::
    println!("{:?}", lookup(&"2001:218::".parse().unwrap()) );
}