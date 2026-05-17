use crate::error::Result;

pub mod pushover;

#[derive(Debug, Clone)]
pub struct Message {
    pub body: String,
    pub title: Option<String>,
    pub priority: Option<i8>,
    pub sound: Option<String>,
    pub device: Option<String>,
    pub url: Option<String>,
    pub url_title: Option<String>,
}

pub trait Service {
    fn send(&self, msg: &Message) -> Result<()>;
}
