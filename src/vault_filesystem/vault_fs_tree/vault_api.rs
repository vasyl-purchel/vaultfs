extern crate fuse;
extern crate libc;
extern crate env_logger;
extern crate time;
extern crate clap;
extern crate reqwest;
extern crate serde_json;

use reqwest::{Client, Method, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Debug)]
pub struct VaultListMetadata {
  pub keys: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct VaultListMetadataResponse {
  pub data: VaultListMetadata,
  pub request_id: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct VaultGetMetadata {
  pub created_time: String,
  pub updated_time: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct VaultGetMetadataResponse {
  pub data: VaultGetMetadata,
  pub request_id: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct VaultGetData {
  pub data: HashMap<String, String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct VaultGetDataResponse {
  pub data: VaultGetData,
  pub request_id: String,
}

pub struct VaultClient {
  vault_address: String,
  vault_token: String,
  client: Client,
}

impl VaultClient {
  pub fn new(address: String, token: String) -> VaultClient {
    return VaultClient {
      vault_address: address,
      vault_token: token,
      client: reqwest::Client::new(),
    }
  }

  fn send(&self, method: &[u8], path: &str) -> Result<Option<String>> {
    let http_method = Method::from_bytes(method).unwrap();

    let uri = format!("{}{}", self.vault_address, path);
    let mut req = self.client.request(http_method, &uri[..]);

    let hdr = reqwest::header::HeaderName::from_lowercase(b"x-vault-token").unwrap();
    let hdr_sv = &self.vault_token[..].trim();
    println!("hdr_sv: '{}'", hdr_sv);
    let hdr_v = reqwest::header::HeaderValue::from_str(&hdr_sv).unwrap();
    req = req.header(hdr, hdr_v);

    let mut resp = req.send()?;
    match resp.status() {
      reqwest::StatusCode::NOT_FOUND => return Ok(None),
      _ => return Ok(Some(resp.text()?)),
    }
  }

  pub fn list_metadata(&self, path: &str) -> serde_json::Result<VaultListMetadataResponse> {
    let data = self.send(b"LIST", format!("/v1/secret/metadata{}", path).as_str());
    match data {
      Ok(Some(text)) => {
        let json_data : VaultListMetadataResponse = serde_json::from_str(&text[..])?;
        return Ok(json_data)
      },
      _ => {
        println!("<{}>failed to get good resp: {:?}", path, data);
        panic!("boom list_metadata");
      },
    }
  }

  pub fn get_metadata(&self, path: &str) -> serde_json::Result<VaultGetMetadataResponse> {
    let data = self.send(b"GET", format!("/v1/secret/metadata{}", path).as_str());
    match data {
      Ok(Some(text)) => {
        let json_data : VaultGetMetadataResponse = serde_json::from_str(&text[..]).unwrap();
        return Ok(json_data);
      },
      _ => {
        println!("<{}>failed to get good resp: {:?}", path, data);
        panic!("boom get_metadata");
      },
    }
  }

  pub fn get_data(&self, path: &str) -> serde_json::Result<VaultGetDataResponse> {
    let data = self.send(b"GET", format!("/v1/secret/data{}", path).as_str());
    match data {
      Ok(Some(text)) => {
        let json_data : VaultGetDataResponse = serde_json::from_str(&text[..]).unwrap();
        return Ok(json_data);
      },
      _ => {
        println!("<{}>failed to get good resp: {:?}", path, data);
        panic!("boom get_data");
      },
    }
  }
}
