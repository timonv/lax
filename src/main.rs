#![feature(io)]

extern crate hyper;
extern crate regex;


// TODO
fn main() {
    let token = authentication::get_oauth_token();
    println!("{}", token);
}

mod authentication {
    // Authentication
    use std::old_io::net::ip::Ipv4Addr;
    use hyper::server::{Server, Request, Response};
    use hyper::uri::RequestUri::AbsolutePath;
    use std::old_io::Command;
    // use hyper::status;
    use regex::Regex;
    use std::sync::mpsc::channel;
    use std::sync::Mutex;

    type AuthCode = String;

    pub fn get_oauth_token() -> AuthCode {
        Command::new("xdg-open").arg("https://slack.com/oauth/authorize?client_id=2334733471.3592055147").output().unwrap();
        let server = Server::http(Ipv4Addr(127, 0, 0, 1), 9999);

        let (tx, rx) = channel();
        let mtx = Mutex::new(tx);

        let mut _guard = server.listen(move |req: Request, res: Response| {
            match req.uri {
                AbsolutePath(path) => {
                    let authcode = extract_auth_code(&path).unwrap();
                    mtx.lock().unwrap().send(authcode).unwrap();
                },
                _ => ()
            }

            let mut res = res.start().unwrap();
            res.write_all(b"Thanks! Please return to Lax").unwrap();
            res.end().unwrap();
        }).unwrap();

        let authcode = rx.recv().unwrap();
        _guard.close().unwrap();
        authcode
    }

    fn extract_auth_code(path: &str) -> Result<AuthCode, &'static str> {
        let re = Regex::new(r"code=(.+?)(&|$)").unwrap();
        match re.captures(path) {
            Some(captures) => Ok(captures.at(1).unwrap().to_string()),
            None => Err("Expected authentication code.")
        }
    }

    #[cfg(test)]
    mod tests {
        use super::extract_auth_code;

        #[test]
        fn test_extract_auth_code() {
            let path = "www.timonv.nl?code=blablabla";
            assert_eq!(extract_auth_code(path).unwrap(), "blablabla".to_string())
        }
    }
}
