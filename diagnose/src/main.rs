use bleecn::TestResult;
use maxminddb::geoip2;
use rocket::http::Status;
use rocket::request::{self, FromRequest, Outcome, Request};
use serde::Serialize;
use slog::Drain;
use std::collections::HashMap;
use std::net::{IpAddr, Ipv4Addr};
use std::str::FromStr;

#[macro_use]
extern crate rocket;

struct ClientIp {
    pub ip: IpAddr,
}

#[derive(Serialize)]
struct DiagnosisResult {
    bleeching: TestResult,
    geo: std::collections::HashMap<IpAddr, String>,
}

fn geotag_ips(ips: Vec<IpAddr>) -> HashMap<IpAddr, String> {
    let mut result = HashMap::<IpAddr, String>::new();

    let open_georeader_result =
        maxminddb::Reader::open_readfile("geolite/GeoLite2-City_20240723/GeoLite2-City.mmdb");

    if open_georeader_result.is_err() {
        return result;
    }

    let georeader = open_georeader_result.unwrap();

    ips.iter().for_each(|ip| {
        result.insert(
            *ip,
            georeader
                .lookup_prefix::<geoip2::City>(*ip)
                .map(|(city, _)| {
                    city.location
                        .map(|location| {
                            format!(
                                "{}, {}",
                                location.latitude.unwrap_or_default(),
                                location.longitude.unwrap_or_default()
                            )
                        }).unwrap_or("NA".to_string())
                }).unwrap_or("NA".to_string())
        );
    });
    result
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for ClientIp {
    type Error = std::io::Error;

    async fn from_request(request: &'r Request<'_>) -> request::Outcome<Self, Self::Error> {
        let keys: Vec<&str> = request.headers().get("X-Real-IP").collect::<Vec<&str>>();
        match keys.len() {
            1 => {
                let client_ip = IpAddr::from_str(keys[0]).unwrap();
                Outcome::Success(ClientIp { ip: client_ip })
            }
            _ => Outcome::Error((
                Status::BadRequest,
                std::io::Error::other("No client IP address in header"),
            )),
        }
    }
}

#[get("/")]
async fn index(client_ip: ClientIp) -> String {
    let log_v: Vec<u8> = vec![];
    let decorator = slog_term::PlainSyncDecorator::new(log_v);
    let drain = slog_term::FullFormat::new(decorator)
        .build()
        .filter_level(slog::Level::Info)
        .fuse();
    let logger = slog::Logger::root(drain, slog::o!("version" => "0.5"));

    let target = match client_ip.ip {
        IpAddr::V4(addr) => bleecn::Target::Ipv4(addr),
        IpAddr::V6(addr) => bleecn::Target::Ipv6(addr),
    };
    let result = bleecn::bleecn(target, 2, Some(10), 2, true, &logger).unwrap();

    let geo = geotag_ips(result.path.hops().unwrap());
    let diagnosis_result = DiagnosisResult {
        bleeching: result,
        geo,
    };

    let output = serde_json::to_string(&diagnosis_result);

    output.unwrap()
}

#[rocket::main]
async fn main() -> Result<(), rocket::Error> {
    let configuration = rocket::Config {
        port: 8888,
        address: std::net::IpAddr::V4(Ipv4Addr::UNSPECIFIED),
        ..Default::default()
    };
    rocket::build()
        .configure(configuration)
        .mount("/", routes![index])
        .launch()
        .await?;
    Ok(())
}
