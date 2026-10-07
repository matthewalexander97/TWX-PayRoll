use super::*;
use std::{
    io::{Read, Write},
    net::TcpListener,
    sync::atomic::{AtomicBool, Ordering},
    time::Instant,
};
#[derive(Clone, Serialize, Deserialize)]
pub struct ClientConfig {
    pub client_id: String,
    pub client_secret: String,
}
#[derive(Serialize, Deserialize)]
pub struct Credential {
    pub client: ClientConfig,
    pub refresh_token: String,
    pub email: String,
}
pub fn client_file(bytes: &[u8]) -> Result<ClientConfig> {
    if bytes.len() > 16_384 {
        return Err("Choose the small Google OAuth Desktop client JSON file.".into());
    }
    let v: Value = serde_json::from_slice(bytes).map_err(|_| "Invalid Google client file")?;
    let installed = &v["installed"];
    let client_id = installed["client_id"]
        .as_str()
        .ok_or("Choose an installed/Desktop OAuth client, not a web or service-account key.")?
        .to_string();
    let client_secret = installed["client_secret"]
        .as_str()
        .ok_or("Missing Desktop client secret")?
        .to_string();
    if !client_id.ends_with(".apps.googleusercontent.com")
        || client_id.len() > 300
        || client_id.chars().any(char::is_control)
        || client_secret.is_empty()
        || client_secret.len() > 300
        || client_secret.chars().any(char::is_control)
    {
        return Err("Invalid Desktop OAuth client configuration".into());
    }
    // Never trust endpoint URLs from a selected credentials file.
    Ok(ClientConfig {
        client_id,
        client_secret,
    })
}
pub fn challenge(verifier: &str) -> String {
    URL_SAFE_NO_PAD.encode(Sha256::digest(verifier.as_bytes()))
}
pub fn authorization(
    client: &ClientConfig,
    service: Service,
    redirect: &str,
    state: &str,
    verifier: &str,
) -> String {
    let mut url = reqwest::Url::parse("https://accounts.google.com/o/oauth2/v2/auth")
        .expect("fixed OAuth URL");
    url.query_pairs_mut().extend_pairs([
        ("client_id", client.client_id.as_str()),
        ("redirect_uri", redirect),
        ("response_type", "code"),
        ("scope", service.scope()),
        ("state", state),
        ("code_challenge", challenge(verifier).as_str()),
        ("code_challenge_method", "S256"),
        ("access_type", "offline"),
        ("prompt", "consent select_account"),
    ]);
    url.to_string()
}
pub fn callback(target: &str, state: &str) -> Result<String> {
    let url = reqwest::Url::parse(&format!("http://127.0.0.1{target}"))
        .map_err(|_| "Invalid OAuth callback")?;
    let pairs = url.query_pairs().collect::<Vec<_>>();
    if url.path() != "/oauth/callback"
        || pairs.iter().filter(|(k, _)| k == "state").count() != 1
        || pairs
            .iter()
            .find(|(k, _)| k == "state")
            .map(|(_, v)| v.as_ref())
            != Some(state)
    {
        return Err("OAuth state did not match; connection canceled.".into());
    }
    if pairs.iter().any(|(k, _)| k == "error") {
        return Err(
            "Google connection canceled or denied. Local workflows remain available.".into(),
        );
    }
    let codes = pairs
        .iter()
        .filter(|(k, _)| k == "code")
        .collect::<Vec<_>>();
    if codes.len() != 1 || codes[0].1.is_empty() || codes[0].1.len() > 4096 {
        return Err("Missing authorization code".into());
    }
    Ok(codes[0].1.to_string())
}
pub fn connect(
    client: ClientConfig,
    service: Service,
    http: &dyn Transport,
    cancel: &AtomicBool,
) -> Result<Credential> {
    let listener = TcpListener::bind(("127.0.0.1", 0))
        .map_err(|_| "Local OAuth callback could not bind. Check local policy.")?;
    listener
        .set_nonblocking(true)
        .map_err(|_| "OAuth listener unavailable")?;
    let redirect = format!(
        "http://127.0.0.1:{}/oauth/callback",
        listener
            .local_addr()
            .map_err(|_| "OAuth listener unavailable")?
            .port()
    );
    let state = Uuid::new_v4().simple().to_string();
    let verifier = format!("{}{}", Uuid::new_v4().simple(), Uuid::new_v4().simple());
    open_authorization(&authorization(
        &client, service, &redirect, &state, &verifier,
    ))?;
    let started = Instant::now();
    let code = loop {
        if cancel.load(Ordering::SeqCst) {
            return Err("Connection canceled. No credential was saved.".into());
        }
        if started.elapsed() > Duration::from_secs(180) {
            return Err("Google connection timed out. Try Connect again.".into());
        }
        match listener.accept() {
            Ok((mut stream, peer)) => {
                if !peer.ip().is_loopback() {
                    continue;
                }
                stream
                    .set_read_timeout(Some(Duration::from_secs(2)))
                    .map_err(|_| "OAuth callback unavailable")?;
                let mut bytes = vec![0u8; 16_384];
                let mut length = 0;
                loop {
                    match stream.read(&mut bytes[length..]) {
                        Ok(0) => break,
                        Ok(n) => {
                            length += n;
                            if bytes[..length].windows(4).any(|v| v == b"\r\n\r\n")
                                || length == bytes.len()
                            {
                                break;
                            }
                        }
                        Err(_) => break,
                    }
                }
                let line = String::from_utf8_lossy(&bytes[..length]);
                let mut parts = line.lines().next().unwrap_or("").split_whitespace();
                let method = parts.next();
                let target = parts.next().unwrap_or("");
                if method != Some("GET") || !target.starts_with("/oauth/callback?") {
                    let _ = stream.write_all(
                        b"HTTP/1.1 404 Not Found\r\nContent-Length: 0\r\nConnection: close\r\n\r\n",
                    );
                    continue;
                }
                let result = callback(target, &state);
                let body="You may close this browser tab and return to TWX Payroll. No report was sent by connecting.";
                let response=format!("HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nCache-Control: no-store\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",body.len());
                let _ = stream.write_all(response.as_bytes());
                break result?;
            }
            Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                std::thread::sleep(Duration::from_millis(50))
            }
            Err(_) => return Err("OAuth callback failed".into()),
        }
    };
    if cancel.load(Ordering::SeqCst) {
        return Err("Connection canceled".into());
    }
    let response = http
        .request(
            Endpoint::Token,
            None,
            Payload::Form(vec![
                ("client_id".into(), client.client_id.clone()),
                ("client_secret".into(), client.client_secret.clone()),
                ("code".into(), code),
                ("code_verifier".into(), verifier),
                ("redirect_uri".into(), redirect),
                ("grant_type".into(), "authorization_code".into()),
            ]),
        )
        .map_err(|e| e.message(false))?;
    let credential = credential_from_response(client, service, response, http)?;
    if cancel.load(Ordering::SeqCst) {
        return Err("Connection canceled. No credential was saved.".into());
    }
    Ok(credential)
}
pub fn credential_from_response(
    client: ClientConfig,
    service: Service,
    response: Value,
    http: &dyn Transport,
) -> Result<Credential> {
    let access = response["access_token"]
        .as_str()
        .filter(|s| !s.is_empty())
        .ok_or("Google returned no access token")?;
    let scopes = response["scope"].as_str().unwrap_or("");
    let required = service.api_scope();
    if !scopes.split_whitespace().any(|v| v == required) {
        return Err("Required Google permission was not granted. Reconnect and approve the selected service.".into());
    }
    let refresh_token = response["refresh_token"]
        .as_str()
        .filter(|s| !s.is_empty())
        .ok_or("Google returned no refresh token. Revoke the old grant and reconnect.")?
        .to_string();
    let identity = http
        .request(Endpoint::Identity, Some(access), Payload::Get)
        .map_err(|e| e.message(false))?;
    let email = identity["email"]
        .as_str()
        .filter(|s| output::recipient(s))
        .ok_or("Connected account did not provide a valid email identity")?
        .to_string();
    if identity["email_verified"] != true {
        return Err("Google account email is not verified".into());
    }
    Ok(Credential {
        client,
        refresh_token,
        email,
    })
}
pub fn access(credential: &Credential, http: &dyn Transport) -> Result<String> {
    let v = http
        .request(
            Endpoint::Token,
            None,
            Payload::Form(vec![
                ("client_id".into(), credential.client.client_id.clone()),
                (
                    "client_secret".into(),
                    credential.client.client_secret.clone(),
                ),
                ("refresh_token".into(), credential.refresh_token.clone()),
                ("grant_type".into(), "refresh_token".into()),
            ]),
        )
        .map_err(|e| e.message(false))?;
    v["access_token"]
        .as_str()
        .filter(|v| !v.is_empty())
        .map(str::to_string)
        .ok_or("Google credential expired or revoked. Reconnect.".into())
}
#[cfg(windows)]
fn open_authorization(url: &str) -> Result<()> {
    use windows::{
        core::PCWSTR,
        Win32::UI::{Shell::ShellExecuteW, WindowsAndMessaging::SW_SHOWNORMAL},
    };
    let wide: Vec<u16> = url.encode_utf16().chain(Some(0)).collect();
    let verb: Vec<u16> = "open".encode_utf16().chain(Some(0)).collect();
    let r = unsafe {
        ShellExecuteW(
            None,
            PCWSTR(verb.as_ptr()),
            PCWSTR(wide.as_ptr()),
            None,
            None,
            SW_SHOWNORMAL,
        )
    };
    if r.0 as isize <= 32 {
        return Err("Default browser could not open Google authorization".into());
    }
    Ok(())
}
#[cfg(not(windows))]
fn open_authorization(_: &str) -> Result<()> {
    Err("Google connection currently requires Windows".into())
}
