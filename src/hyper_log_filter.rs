use log::Record;
use log4rs::{
    config::{Deserialize, Deserializers},
    filter::{Filter, Response},
};

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug, serde::Deserialize)]
pub struct HyperFilterConfig {}

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug, Default)]
pub struct HyperFilter {}

impl Filter for HyperFilter {
    fn filter(&self, record: &Record) -> Response {
        if record.module_path().unwrap_or_default().starts_with("hyper") {
            Response::Reject
        } else {
            Response::Neutral
        }
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub struct HyperFilterDeserializer;

impl Deserialize for HyperFilterDeserializer {
    type Trait = dyn Filter;

    type Config = HyperFilterConfig;

    fn deserialize(&self, _: Self::Config, _: &Deserializers) -> anyhow::Result<Box<dyn Filter>> {
        Ok(Box::<HyperFilter>::default())
    }
}
