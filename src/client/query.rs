use chrono::{NaiveDate, NaiveDateTime, NaiveTime};
use enumflags2::{bitflags, BitFlags};
use lazy_regex::regex;
use regex::Regex;
use scraper::{Html, Selector};

use crate::{error::BahnResult, BahnClient, Line, Station, ToApiType};

#[derive(Clone, Debug)]
pub struct TimeTableQuery {
    pub stop: Station,
    pub board_type: BoardType,
    pub products: BitFlags<ProductType>,
    pub time: NaiveTime,
    pub date: NaiveDate,
    pub limit: usize,
}

#[derive(Clone, Debug)]
pub enum BoardType {
    Arrival,
    Departure,
}

impl ToApiType for BoardType {
    type ApiType = &'static str;

    fn to_api_type(&self) -> &'static str {
        match self {
            BoardType::Arrival => "arr",
            BoardType::Departure => "dep",
        }
    }
}

#[bitflags]
#[repr(u16)]
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum ProductType {
    ICE = 0b1000000000,
    IC = 0b0100000000,
    InterRegio = 0b0010000000,
    Regio = 0b0001000000,
    SBahn = 0b0000100000,
    Bus = 0b0000010000,
    Ship = 0b0000001000,
    Metro = 0b0000000100,
    Tram = 0b0000000010,
    Taxi = 0b0000000001,
}

impl ToApiType for BitFlags<ProductType> {
    type ApiType = String;

    fn to_api_type(&self) -> Self::ApiType {
        format!("{:b}", self.bits())
    }
}

#[derive(Clone, Debug)]
pub struct ConnectionEntry {
    /// The line
    pub line: Line,

    /// The station this train terminates at
    pub end: Station,

    /// Planned arrival time
    pub plan_arrival: NaiveTime,

    /// Expected arrival time after taking delay into account
    pub exp_arrival: NaiveTime,

    /// An url with detailed information about this connection
    details_url: String,
}

lazy_static::lazy_static! {
    static ref LIST_ELEMENTS_SELECTOR: Selector = Selector::parse("div.sqdetails").unwrap();
    static ref CONNECTION_DATA_SELECTOR: Selector = Selector::parse("a").unwrap();
    static ref LINE_DATA_REGEX: Regex = Regex::new(
        r"(?P<line>.+)\s+(>>|<<)\s+(?P<end>[^\n]+)\n+(?P<plan_arr>\d{2}:\d{2})\n+(?P<exp_arr>\d{2}:\d{2})").unwrap();
}

impl BahnClient {
    pub async fn query(&self, query: TimeTableQuery) -> BahnResult<Vec<ConnectionEntry>> {
        let response_text = self
            .client
            .get(&self.bhtafel_url)
            .query(&[
                ("bt", query.board_type.to_api_type().to_owned()),
                ("si", query.stop.0),
                ("p", query.products.to_api_type()),
                ("max", query.limit.to_string()),
                ("rt", String::from("1")),
                ("start", String::from("1")),
            ])
            .send()
            .await?
            .text()
            .await?;

        let html = Html::parse_document(&response_text);

        let connections = html
            .select(&LIST_ELEMENTS_SELECTOR)
            .map(|e| {
                // TODO: Don't unwrap
                let line_data = e.text().fold(String::new(), |a, b| format!("{a}\n{b}"));
                // TODO: Don't unwrap
                let connection_data = e
                    .select(&CONNECTION_DATA_SELECTOR)
                    .next()
                    .expect("Missing connection data");
                let url = connection_data
                    .value()
                    .attr("href")
                    .expect("Missing data url");
                let capts = (&*LINE_DATA_REGEX).captures(&line_data).unwrap();
                let plan_arrival = NaiveTime::parse_from_str(&capts["plan_arr"], "%H:%M").unwrap();
                let exp_arrival = NaiveTime::parse_from_str(&capts["exp_arr"], "%H:%M").unwrap();
                let line_name = &capts["line"];
                let line_name = regex!(r" +").replace_all(line_name, " ");

                ConnectionEntry {
                    line: line_name.into(),
                    end: capts["end"].into(),
                    plan_arrival,
                    exp_arrival,
                    details_url: url.to_owned(),
                }
            })
            .collect::<Vec<_>>();

        Ok(connections)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{client::query::TimeTableQuery, BahnClient, Station};
    use chrono::Utc;
    use enumflags2::BitFlags;

    #[tokio::test]
    async fn it_queries_timetables() {
        let client = BahnClient::default();
        let entries = client
            .query(TimeTableQuery {
                stop: Station("Hackescher Markt (S), Berlin".into()),
                board_type: BoardType::Arrival,
                products: BitFlags::<ProductType>::ALL,
                time: Utc::now().time(),
                date: Utc::now().date_naive(),
                limit: 10,
            })
            .await
            .unwrap();
        println!("{entries:#?}");
        assert!(
            entries.is_empty() == false,
            "Hackescher Markt has trains arriving"
        )
    }
}
