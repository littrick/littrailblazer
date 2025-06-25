use anyhow::{Result, ensure};
use bytes::Bytes;
use reqwest::StatusCode;
use url::Url;

pub fn download(url: &Url) -> Result<Bytes> {
    ensure!(
        url.scheme() == "http" || url.scheme() == "https",
        "{} is not a http url",
        url.as_str()
    );

    let resp = reqwest::blocking::get(url.clone())?;

    ensure!(
        resp.status() == StatusCode::OK,
        "Download file fail: status code: {}",
        resp.status()
    );

    let contents = resp.bytes()?;

    Ok(contents)
}
