#![cfg_attr(windows, feature(abi_vectorcall))]
use std::{net::IpAddr, str::FromStr};

use ext_php_rs::prelude::*;

use fastly;

#[php_function]
pub fn fastly_geo_lookup(ip: String) -> FastlyGeo {
    let ip = IpAddr::from_str(ip.as_str()).unwrap();
    let geo = fastly::geo::geo_lookup(ip).unwrap();

    FastlyGeo {
        city: geo.city().to_string(),
    }
}

#[php_class(name = "FastlyGeo")]
pub struct FastlyGeo {
    city: String,
}

// #[php_impl]
// impl FastlyGeo {
//     pub fn geo_lookup(ip: String) -> FastlyGeo {
//         let ip = IpAddr::from_str(ip.as_str()).unwrap();
//         let geo = fastly::geo::geo_lookup(ip).unwrap();

//         Self {
//             city: geo.city().to_string(),
//         }
//     }
// }

#[php_module]
pub fn get_module(module: ModuleBuilder) -> ModuleBuilder {
    module
}
