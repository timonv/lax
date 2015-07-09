// Authentication
use hyper::client::{Client};
use hyper::server::{Server, Request, Response, Listening};
use hyper::uri::RequestUri::AbsolutePath;
use regex::Regex;
use rustc_serialize::json::Json;
use std::fs;
use std::io::prelude::*;
use std::process::Command;
use std::sync::Mutex;
use std::sync::mpsc::channel;

pub type AuthToken = String;
type TempCode = String;

#[allow(dead_code)]
const AUTH_TOKEN_FILE: &'static str = "auth_token";
#[allow(dead_code)]
const SLACK_CLIENT_ID: &'static str = "2334733471.3592055147";
#[allow(dead_code)]
const SLACK_CLIENT_SECRET: &'static str = "37721a57d17018b206fb1264caa7d707";

pub fn get_oauth_token_or_panic() -> (AuthToken, Option<Listening>) {
    match maybe_existing_token(AUTH_TOKEN_FILE) {
        Some(token) => (token, None),
        None => {
            let (token, listener) = arrange_new_token();
            store_token(&token);
            (token, Some(listener))
        }
    }
}

fn maybe_existing_token(token_file: &str) -> Option<AuthToken> {
    match fs::File::open(token_file) {
        Ok(mut file) => {
            let mut s = String::new();
            file.read_to_string(&mut s).unwrap();

            if !s.is_empty() {
                Some(s)
            } else {
                None }
        },
        Err(_) => None
    }
}

fn arrange_new_token() -> (AuthToken, Listening) {
    let (temp_code, listener) = request_temp_code();
    let token = request_token(&temp_code);
    (token, listener)
}

// TODO Test
fn store_token(token: &AuthToken) {
    let mut f = fs::File::create(AUTH_TOKEN_FILE).unwrap();
    f.write_all(token.as_bytes()).unwrap();
}

#[allow(dead_code)] // Hard to test
fn request_temp_code() -> (TempCode, Listening) {
    Command::new("xdg-open").arg(format!("https://slack.com/oauth/authorize?scope=client&client_id={}", SLACK_CLIENT_ID)).output().unwrap();

    let (tx, rx) = channel();
    let mtx = Mutex::new(tx);

    let mut guard = Server::http("127.0.0.1:9999").unwrap().handle(move |req: Request, res: Response| {
        match req.uri {
            AbsolutePath(ref path) => {
                match extract_temp_code(&path) {
                    Some(tempcode) =>  {
                        mtx.lock().unwrap().send(tempcode).unwrap();
                    },
                    None => ()
                }
            },
            _ => ()
        }

        let mut res = res.start().unwrap();
        res.write_all(b"Thanks! Please return to Lax").unwrap();
        res.end().unwrap();
    }).unwrap();

    let tempcode = rx.recv().unwrap();
    guard.close().unwrap();
    (tempcode, guard)
}

fn request_token(temp_code: &TempCode) -> AuthToken {
    let mut client = Client::new();
    // I thought & is sufficient to make it a slice
    let mut res = client.get(format_access_uri(temp_code).as_str()).send().unwrap();
    let mut body = String::new();
    res.read_to_string(&mut body).unwrap();
    match Json::from_str(&body) {
        Ok(json) => {
            match json.find("access_token") {
                Some(j) => j.as_string().unwrap().to_string(),
                _ =>  panic!("Unexpected json in slack response\n{}", json.pretty())
            }
        },
        _ => panic!("Reponse not json")
    }
}

// TODO Needs test
fn format_access_uri(temp_code: &TempCode) -> String {
    let base = "https://slack.com/api/oauth.access";
    let query = format!("?client_id={}&client_secret={}&code={}", SLACK_CLIENT_ID, SLACK_CLIENT_SECRET, temp_code);
    format!("{}{}", base, query)
}

fn extract_temp_code(path: &str) -> Option<TempCode> {
    let re = Regex::new(r"code=(.+?)(&|$)").unwrap();
    re.captures(path).map(|cap| cap.at(1).unwrap().to_string())
}

#[cfg(test)]
mod tests {
    use super::extract_temp_code;
    use super::maybe_existing_token;
    use std::fs;
    use std::io::Write;
    const AUTH_TOKEN_FILE: &'static str = "auth_token_test";

    #[test]
    fn test_extract_temp_code() {
        let path = "www.timonv.nl?code=blablabla";
        assert_eq!(extract_temp_code(path).unwrap(), "blablabla".to_string())
    }

    #[test]
    fn test_maybe_existing_token() {
        // None if file doesn't exist
        fs::remove_file(AUTH_TOKEN_FILE).unwrap_or(());
        assert_eq!(maybe_existing_token(AUTH_TOKEN_FILE), None);

        // Some if file exists
        let mut f = fs::File::create(AUTH_TOKEN_FILE).unwrap();
        f.write_all(b"123").unwrap();

        assert_eq!(maybe_existing_token(AUTH_TOKEN_FILE), Some("123".to_string()));

        // None if file exists but empty
        let mut f = fs::File::create(AUTH_TOKEN_FILE).unwrap();
        f.write_all(b"").unwrap();

        assert_eq!(maybe_existing_token(AUTH_TOKEN_FILE), None);

        // Cleanup
        fs::remove_file(AUTH_TOKEN_FILE).unwrap_or(());
    }
}
