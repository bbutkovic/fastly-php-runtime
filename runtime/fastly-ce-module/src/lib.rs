use ext_php_rs::{
    convert::{FromZval, IntoZval},
    prelude::*,
};
use serde_variant::to_variant_name;

// todo: using a "mod" fails because ext-php-rs proc macros use global state?
// for now we're writing everything in one module...
// include!("geo.rs");

use std::{net::IpAddr, str::FromStr};

use fastly::geo::{ConnSpeed, Geo};

// #[php_function]
// pub fn fastlyce_geo_lookup(ip: String) -> FastlyGeo {
//     let ip = IpAddr::from_str(ip.as_str()).unwrap();
//     let geo = fastly::geo::geo_lookup(ip).unwrap();

//     FastlyGeo { geo }
// }

#[php_class(name = "FastlyCE\\Geo")]
pub struct FastlyGeo {
    geo: Geo,
}

#[php_impl]
impl FastlyGeo {
    #[getter]
    pub fn get_as_name(&self) -> String {
        self.geo.as_name().to_string()
    }

    #[getter]
    pub fn get_as_number(&self) -> u32 {
        self.geo.as_number()
    }

    #[getter]
    pub fn get_area_code(&self) -> u16 {
        self.geo.area_code()
    }

    #[getter]
    pub fn get_city(&self) -> String {
        self.geo.city().to_string()
    }

    #[getter]
    pub fn get_conn_speed(&self) -> String {
        to_variant_name(&self.geo.conn_speed()).unwrap().to_string()
    }

    #[getter]
    pub fn get_conn_type(&self) -> String {
        to_variant_name(&self.geo.conn_type()).unwrap().to_string()
    }

    #[getter]
    pub fn get_continent(&self) -> String {
        to_variant_name(&self.geo.continent()).unwrap().to_string()
    }

    #[getter]
    pub fn get_country_code(&self) -> String {
        self.geo.country_code().to_string()
    }

    #[getter]
    pub fn get_country_code3(&self) -> String {
        self.geo.country_code3().to_string()
    }

    #[getter]
    pub fn get_country_name(&self) -> String {
        self.geo.country_name().to_string()
    }

    #[getter]
    pub fn get_latitude(&self) -> f64 {
        self.geo.latitude()
    }

    #[getter]
    pub fn get_longitude(&self) -> f64 {
        self.geo.longitude()
    }

    #[getter]
    pub fn get_metro_code(&self) -> i64 {
        self.geo.metro_code()
    }

    #[getter]
    pub fn get_postal_code(&self) -> String {
        self.geo.postal_code().to_string()
    }

    #[getter]
    pub fn get_proxy_description(&self) -> String {
        to_variant_name(&self.geo.proxy_description())
            .unwrap()
            .to_string()
    }

    #[getter]
    pub fn get_proxy_type(&self) -> String {
        to_variant_name(&self.geo.proxy_type()).unwrap().to_string()
    }

    #[getter]
    pub fn get_region(&self) -> Option<String> {
        self.geo.region().map(|r| r.to_string())
    }

    #[getter]
    pub fn get_utc_offset(&self) -> Option<String> {
        self.geo
            .utc_offset()
            .map(|offset| to_variant_name(&offset).unwrap().to_string())
    }
}

#[php_module]
pub fn get_module(module: ModuleBuilder) -> ModuleBuilder {
    module
}
