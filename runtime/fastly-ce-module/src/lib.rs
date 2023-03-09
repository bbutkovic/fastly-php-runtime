use ext_php_rs::prelude::*;
use fastly::{geo::Geo, Request, Response};
use serde_variant::to_variant_name;
use std::{net::IpAddr, str::FromStr};

// todo: using a "mod" fails because ext-php-rs proc macros use global state?
// for now we're writing everything in one module...
// include!("geo.rs");

// Fastly Geolocation

#[php_class(name = "FastlyCE\\Geo")]
pub struct FastlyGeo(Geo);

#[php_impl]
impl FastlyGeo {
    pub fn lookup(ip: String) -> PhpResult<Self> {
        let ip = IpAddr::from_str(ip.as_str()).map_err(|_| "invalid IP address")?;

        let geo = fastly::geo::geo_lookup(ip).ok_or("address lookup failed")?;
        Ok(Self(geo))
    }

    #[getter]
    pub fn get_as_name(&self) -> String {
        self.0.as_name().to_string()
    }

    #[getter]
    pub fn get_as_number(&self) -> u32 {
        self.0.as_number()
    }

    #[getter]
    pub fn get_area_code(&self) -> u16 {
        self.0.area_code()
    }

    #[getter]
    pub fn get_city(&self) -> String {
        self.0.city().to_string()
    }

    #[getter]
    pub fn get_conn_speed(&self) -> String {
        to_variant_name(&self.0.conn_speed()).unwrap().to_string()
    }

    #[getter]
    pub fn get_conn_type(&self) -> String {
        to_variant_name(&self.0.conn_type()).unwrap().to_string()
    }

    #[getter]
    pub fn get_continent(&self) -> String {
        to_variant_name(&self.0.continent()).unwrap().to_string()
    }

    #[getter]
    pub fn get_country_code(&self) -> String {
        self.0.country_code().to_string()
    }

    #[getter]
    pub fn get_country_code3(&self) -> String {
        self.0.country_code3().to_string()
    }

    #[getter]
    pub fn get_country_name(&self) -> String {
        self.0.country_name().to_string()
    }

    #[getter]
    pub fn get_latitude(&self) -> f64 {
        self.0.latitude()
    }

    #[getter]
    pub fn get_longitude(&self) -> f64 {
        self.0.longitude()
    }

    #[getter]
    pub fn get_metro_code(&self) -> i64 {
        self.0.metro_code()
    }

    #[getter]
    pub fn get_postal_code(&self) -> String {
        self.0.postal_code().to_string()
    }

    #[getter]
    pub fn get_proxy_description(&self) -> String {
        to_variant_name(&self.0.proxy_description())
            .unwrap()
            .to_string()
    }

    #[getter]
    pub fn get_proxy_type(&self) -> String {
        to_variant_name(&self.0.proxy_type()).unwrap().to_string()
    }

    #[getter]
    pub fn get_region(&self) -> Option<String> {
        self.0.region().map(|r| r.to_string())
    }

    #[getter]
    pub fn get_utc_offset(&self) -> Option<String> {
        self.0
            .utc_offset()
            .map(|offset| to_variant_name(&offset).unwrap().to_string())
    }
}

// -- Fastly Geolocation

// Fastly Response

#[php_class(name = "FastlyCE\\Response")]
pub struct FastlyResponse(Option<Response>);

#[php_impl]
impl FastlyResponse {
    pub fn into_body_str(&mut self) -> String {
        let response = self.0.take().unwrap();

        response.into_body_str()
    }
}

impl From<Response> for FastlyResponse {
    fn from(value: Response) -> Self {
        Self(Some(value))
    }
}

// -- Fastly Response

// Fastly Request

#[php_class(name = "FastlyCE\\Request")]
pub struct FastlyRequest(Option<Request>);

#[php_impl]
impl FastlyRequest {
    pub fn __construct(method: String, url: String) -> Self {
        Self(Some(Request::new(method, url)))
    }

    pub fn get(url: String) -> Self {
        Self::__construct("GET".to_string(), url)
    }

    pub fn head(url: String) -> Self {
        Self::__construct("HEAD".to_string(), url)
    }

    pub fn post(url: String) -> Self {
        Self::__construct("POST".to_string(), url)
    }

    pub fn put(url: String) -> Self {
        Self::__construct("PUT".to_string(), url)
    }

    pub fn delete(url: String) -> Self {
        Self::__construct("DELETE".to_string(), url)
    }

    pub fn connect(url: String) -> Self {
        Self::__construct("CONNECT".to_string(), url)
    }

    pub fn with_header(&mut self, name: String, value: String) -> Self {
        let request = self.0.take().unwrap();

        let request = request.with_header(name, value);
        Self(Some(request))
    }

    pub fn with_body(&mut self, body: String) -> Self {
        let request = self.0.take().unwrap();

        let request = request.with_body(body);
        Self(Some(request))
    }

    pub fn send(&mut self, backend: String) -> PhpResult<FastlyResponse> {
        let request = self.0.take().unwrap();

        request
            .send(backend)
            .map(FastlyResponse::from)
            .map_err(|err| PhpException::default(err.to_string()))
    }
}

// -- Fastly Request

#[php_module]
pub fn get_module(module: ModuleBuilder) -> ModuleBuilder {
    module
}
