//! Simple GTFS server for an embedded departure board system.
//!
//! This program acts as a hub, allowing multiple systems to request gtfs information to display. (todo! async/thread based based system)
//! The server accepts a request from the remote device, specifying the gtfs data required, such as
//! (in the case of a departure board) stop(s) to be displayed, the (maximum) number of upcoming
//! services to return, an optional direction for services and desired optional fields, such as
//! scheduled arrival time, expected arrival time, vehicle type (e.g. NGR, IMU etc. for trains),
//! with certain mandatory fields such as a route identifier.
//!
//! The communication protocol is defined within the gtfs-requests.proto file.
//!
//! Future work:
//!     - Extend request/response types to more than departure board information.
//!       Could easily extend to respond with fields such as location, as well as provide location
//!       tracking for connections and vehicle services, by receiving position reports from
//!       embedded devices and returning the closest services/services within x distance, allowing
//!       the embedded device to compute closest vehicle/vehicle most likely to be closest.
//!     - Extend the program to accept a real-time feed url and static database path to allow the
//!       server to process non-seq requests, which will be the default feed for development.
//!     - Add protobuf compilation to build.rs

use gtfs_server::gtfs::gtfs_real_time as rt;
use gtfs_server::gtfs::gtfs_real_time::FeedType;
use std::error::Error;

const _GTFS_RT_URL: &str = "https://gtfsrt.api.translink.com.au/api/realtime/SEQ";
const GTFS_RT_TRIP_UPDATE_URL: &str =
    "https://gtfsrt.api.translink.com.au/api/realtime/SEQ/TripUpdates";
const GTFS_RT_VEHICLE_POSITIONS_URL: &str =
    "https://gtfsrt.api.translink.com.au/api/realtime/SEQ/VehiclePositions";
const GTFS_RT_ALERTS_URL: &str = "https://gtfsrt.api.translink.com.au/api/realtime/SEQ/Alerts";

#[tokio::main]
pub async fn main() -> Result<(), Box<dyn Error>> {
    println!("Hello, world!");

    // let resp = rt::from_url(GTFS_RT_URL).await?;

    let mut rt = rt::GtfsRt::new(
        GTFS_RT_TRIP_UPDATE_URL,
        GTFS_RT_VEHICLE_POSITIONS_URL,
        GTFS_RT_ALERTS_URL,
    );

    // Establish connection to real-time feed to verify validity/expiry of gtfs static server.
    // Establish connection to static database.

    // Setup thread-pool/async based http server, ready to accept incoming connections

    loop {
        // Establish a connection to incoming requests, reading provided requests and responding with
        // proto files containing processed requests.

        let latest_fm = rt.latest(FeedType::VehiclePosition).await?;
        println!("Latest header\n{:?}", latest_fm.header);
    }
}
