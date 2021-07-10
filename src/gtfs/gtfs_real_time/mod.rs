//! GTFS real time manager, such as connecting to real-time server and returning fields.
//!
//! Requires GTFS-RT feed to supply timestamp, otherwise need to manually wait for new feeds or block normally todo! -> call update instead?

use prost::{DecodeError, Message};
use reqwest;
use reqwest::Error;
use std::fmt::Formatter;
use tokio::time::{sleep, Duration};

// GTFS-RT definitions.
include!(concat!(env!("OUT_DIR"), "/gtf_sv2.realtime.rs"));

#[derive(Debug)]
pub enum FeedType {
    TripUpdate,
    VehiclePosition,
    Alert,
}

impl std::fmt::Display for FeedType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        use FeedType::*;
        match self {
            TripUpdate => write!(f, "\"Trip Update\""),
            VehiclePosition => write!(f, "\"Vehicle Position\""),
            Alert => write!(f, "\"Alert\""),
        }
    }
}

#[derive(Debug)]
pub enum GtfsRtError {
    ProtobufDecodeError(prost::DecodeError),
    DownloadError(reqwest::Error),
    MissingFeed(FeedType),
}

impl std::error::Error for GtfsRtError {}

impl std::fmt::Display for GtfsRtError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        use GtfsRtError::*;
        match self {
            ProtobufDecodeError(e) => write!(f, "Error decoding protocol-buffer\n{:}", e),
            DownloadError(e) => write!(f, "Error obtaining real-time feed from url\n{:}", e),
            MissingFeed(feed) => write!(f, "Cannot query {:} feed, missing feed url", feed),
        }
    }
}

impl From<reqwest::Error> for GtfsRtError {
    fn from(e: Error) -> Self {
        GtfsRtError::DownloadError(e)
    }
}

impl From<prost::DecodeError> for GtfsRtError {
    fn from(e: DecodeError) -> Self {
        GtfsRtError::ProtobufDecodeError(e)
    }
}

pub struct GtfsRt {
    // rt_url: String, // todo: it is possible this link has only the vehicle positions
    trip_updates_url: Option<String>,
    vehicle_positions_url: Option<String>,
    alerts_url: Option<String>,
    latest_timestamp: u64,
    // how long to wait for between querying feed for new data (optional)
}

impl GtfsRt {
    /// Create a new GTFS-RT instance for a single-url RT feed.
    /// Assumes the single-url RT feed contains vehicle positions. If a different feed is supplied,
    /// create a new instance from multiple urls and set the unused feeds to ```None```.
    ///
    /// # Example
    ///
    /// ```
    /// use gtfs_server::gtfs::gtfs_real_time as rt;
    /// let gtfs_rt = rt::GtfsRt::new_single_url(URL);
    /// ```
    pub fn new_single_url(rt_url: &str) -> Self {
        GtfsRt {
            trip_updates_url: None,
            vehicle_positions_url: Some(String::from(rt_url)),
            alerts_url: None,
            latest_timestamp: 0,
        }
    }

    /// Create a new GTFS-RT instance with all feed types (trip updates, vehicle positions, alerts)
    /// supplied.
    ///
    /// # Example
    ///
    /// ```
    /// use gtfs_server::gtfs::gtfs_real_time as rt;
    /// let gtfs_rt = rt::GtfsRt::new(TRIP_UPDATES_URL, VEHICLE_POSITIONS_URL, ALERTS_URL);
    /// ```
    pub fn new(trip_updates_url: &str, vehicle_positions_url: &str, alerts_url: &str) -> Self {
        GtfsRt {
            trip_updates_url: Some(String::from(trip_updates_url)),
            vehicle_positions_url: Some(String::from(vehicle_positions_url)),
            alerts_url: Some(String::from(alerts_url)),
            latest_timestamp: 0,
        }
    }

    /// Create a new GTFS-RT instance with any combination of feed types (trip updates, vehicle
    /// positions, alerts) supplied.
    ///
    /// # Example
    ///
    /// ```
    /// use gtfs_server::gtfs::gtfs_real_time as rt;
    /// let gtfs_rt = rt::GtfsRt::new(TRIP_UPDATES_URL, VEHICLE_POSITIONS_URL, ALERTS_URL);
    /// ```
    pub fn new_optional(
        trip_updates_url: Option<&str>,
        vehicle_positions_url: Option<&str>,
        alerts_url: Option<&str>,
    ) -> Self {
        GtfsRt {
            trip_updates_url: trip_updates_url.map(|s| String::from(s)),
            vehicle_positions_url: vehicle_positions_url.map(|s| String::from(s)),
            alerts_url: alerts_url.map(|s| String::from(s)),
            latest_timestamp: 0,
        }
    }

    async fn _update(url: &String) -> Result<FeedMessage, GtfsRtError> {
        let bytes = reqwest::get(url).await?.bytes().await?;
        let mut buf: &[u8] = &bytes[..];
        Ok(FeedMessage::decode(&mut buf)?)
    }

    /// Retrieve the latest trip update dataset.
    ///
    /// # Example
    ///
    /// ```
    /// use gtfs_server::gtfs::gtfs_real_time as rt;
    /// let gtfs_rt = rt::GtfsRt::new(TRIP_UPDATES_URL, VEHICLE_POSITIONS_URL, ALERTS_URL);
    /// let trip_update_feed = gtfs_rt.update_trip_updates()?;
    ///
    /// for trip_update_entity in trip_update_feed.entities {
    ///     println!("{:?}", trip_update_entity);
    /// }
    /// ```
    pub async fn update_trip_updates(&self) -> Result<FeedMessage, GtfsRtError> {
        match &self.trip_updates_url {
            None => Err(GtfsRtError::MissingFeed(FeedType::TripUpdate)),
            Some(url) => GtfsRt::_update(url).await,
        }
    }

    /// Retrieve the latest vehicle_positions dataset.
    ///
    /// # Example
    ///
    /// ```
    /// use gtfs_server::gtfs::gtfs_real_time as rt;
    /// let gtfs_rt = rt::GtfsRt::new(TRIP_UPDATES_URL, VEHICLE_POSITIONS_URL, ALERTS_URL);
    /// let vehicle_positions_feed = gtfs_rt.update_vehicle_positions()?;
    ///
    /// for vehicle_position_entity in vehicle_positions_feed.entities {
    ///     println!("{:?}", vehicle_position_entity);
    /// }
    /// ```
    pub async fn update_vehicle_positions(&self) -> Result<FeedMessage, GtfsRtError> {
        match &self.vehicle_positions_url {
            None => Err(GtfsRtError::MissingFeed(FeedType::VehiclePosition)),
            Some(url) => GtfsRt::_update(url).await,
        }
    }

    /// Retrieve the latest alerts dataset.
    ///
    /// # Example
    ///
    /// ```
    /// use gtfs_server::gtfs::gtfs_real_time as rt;
    /// let gtfs_rt = rt::GtfsRt::new(TRIP_UPDATES_URL, VEHICLE_POSITIONS_URL, ALERTS_URL);
    /// let alerts_feed = gtfs_rt.update_alerts()?;
    ///
    /// for alert_entity in alerts_feed.entities {
    ///     println!("{:?}", alert_entity);
    /// }
    /// ```
    pub async fn update_alerts(&self) -> Result<FeedMessage, GtfsRtError> {
        match &self.alerts_url {
            None => Err(GtfsRtError::MissingFeed(FeedType::Alert)),
            Some(url) => GtfsRt::_update(url).await,
        }
    }

    /// Wait for a new set of data to arrive.
    /// Block on this function to only retrieve more data when new data is available.
    /// If GTFS-RT feed does not supply timestamp then obtains and returns most recent data.
    pub async fn latest(&mut self, feed_type: FeedType) -> Result<FeedMessage, GtfsRtError> {
        loop {
            let fm = match feed_type {
                FeedType::TripUpdate => self.update_trip_updates().await?,
                FeedType::VehiclePosition => self.update_vehicle_positions().await?,
                FeedType::Alert => self.update_alerts().await?,
            };
            let feed_header = &fm.header;

            match feed_header.timestamp {
                None => return Ok(fm),
                Some(time) => {
                    let prev_time = self.latest_timestamp;
                    self.latest_timestamp = time;
                    if time > prev_time {
                        return Ok(fm);
                    }
                }
            };

            sleep(Duration::from_secs(5)).await;
        }
    }
}

/// Retrieve a FeedMessage item from a real time feed url.
pub async fn feed_message_from_url(url: &str) -> Result<FeedMessage, GtfsRtError> {
    let bytes = reqwest::get(url).await?.bytes().await?;

    let mut buf: &[u8] = &bytes[..];
    Ok(FeedMessage::decode(&mut buf)?)
}
