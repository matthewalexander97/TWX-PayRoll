use super::*;
use std::io::Read;
#[derive(Clone, Copy)]
pub enum Endpoint {
    Token,
    Revoke,
    Identity,
    Sheets,
    Gmail,
}
impl Endpoint {
    pub fn url(self) -> &'static str {
        match self {
            Self::Token => "https://oauth2.googleapis.com/token",
            Self::Revoke => "https://oauth2.googleapis.com/revoke",
            Self::Identity => "https://openidconnect.googleapis.com/v1/userinfo",
            Self::Sheets => "https://sheets.googleapis.com/v4/spreadsheets",
            Self::Gmail => "https://gmail.googleapis.com/gmail/v1/users/me/messages/send",
        }
    }
}
pub enum Payload {
    Form(Vec<(String, String)>),
    Json(Value),
    Get,
}
pub trait Transport: Send {
    fn request(
        &self,
        endpoint: Endpoint,
        bearer: Option<&str>,
        payload: Payload,
    ) -> std::result::Result<Value, Failure>;
}
pub struct Https;
impl Transport for Https {
    fn request(
        &self,
        endpoint: Endpoint,
        bearer: Option<&str>,
        payload: Payload,
    ) -> std::result::Result<Value, Failure> {
        // Fixed endpoints, TLS verification, no redirects or automatic HTTP retries.
        let c = reqwest::blocking::Client::builder()
            .https_only(true)
            .redirect(reqwest::redirect::Policy::none())
            .retry(reqwest::retry::never())
            .connect_timeout(Duration::from_secs(10))
            .timeout(Duration::from_secs(30))
            .build()
            .map_err(|_| Failure::Unavailable)?;
        let r = match payload {
            Payload::Get => c.get(endpoint.url()),
            Payload::Form(p) => c.post(endpoint.url()).form(&p),
            Payload::Json(p) => c.post(endpoint.url()).json(&p),
        };
        let r = if let Some(token) = bearer {
            r.bearer_auth(token)
        } else {
            r
        };
        let response = r.send().map_err(|_| Failure::Uncertain)?;
        let code = response.status().as_u16();
        if !(200..300).contains(&code) {
            return Err(match code {
                400 | 401 | 403 => Failure::Rejected,
                429 => Failure::RateLimited,
                _ => Failure::Uncertain,
            });
        }
        let mut bytes = Vec::new();
        response
            .take(1_048_577)
            .read_to_end(&mut bytes)
            .map_err(|_| Failure::Uncertain)?;
        if bytes.len() > 1_048_576 {
            return Err(Failure::Uncertain);
        }
        if bytes.is_empty() {
            return Ok(json!({}));
        }
        serde_json::from_slice(&bytes).map_err(|_| Failure::Uncertain)
    }
}
