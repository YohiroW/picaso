use super::*;
use std::{
    io::{BufRead, BufReader, Read, Write},
    net::TcpListener,
    thread,
};

fn serve(status: u16, body: String) -> (OpenAi, thread::JoinHandle<(String, Vec<u8>)>) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let endpoint = format!("http://{}/v1/images", listener.local_addr().unwrap());
    let handle = thread::spawn(move || {
        let (mut socket, _) = listener.accept().unwrap();
        let mut reader = BufReader::new(&socket);
        let mut header = String::new();
        loop {
            let mut line = String::new();
            assert!(reader.read_line(&mut line).unwrap() > 0);
            if line == "\r\n" {
                break;
            }
            header.push_str(&line);
        }
        let length: usize = header
            .lines()
            .filter_map(|l| l.split_once(':'))
            .find(|(k, _)| k.eq_ignore_ascii_case("content-length"))
            .unwrap()
            .1
            .trim()
            .parse()
            .unwrap();
        let mut data = vec![0; length];
        reader.read_exact(&mut data).unwrap();
        write!(socket,"HTTP/1.1 {status} Reply\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",body.len()).unwrap();
        (header, data)
    });
    let mut provider = OpenAi::new("fictional-http-key".into()).unwrap();
    provider.client = Client::builder().no_proxy().build().unwrap();
    provider.test_origin = Some(endpoint);
    (provider, handle)
}
#[test]
fn sends_json_or_one_original_multipart_to_the_official_paths() {
    let png = include_bytes!("../../../icons/icon.png");
    for original in [None, Some(png.as_slice())] {
        let (provider,server)=serve(200,serde_json::json!({"data":[{"b64_json":STANDARD.encode(png)}],"usage":{"total_tokens":23}}).to_string());
        let output = provider
            .render("Clay P icon", original)
            .map_err(|e| e.message)
            .unwrap();
        let (header, data) = server.join().unwrap();
        assert!(header
            .to_ascii_lowercase()
            .contains("authorization: bearer fictional-http-key"));
        assert_eq!(output.images, vec![png.to_vec()]);
        assert_eq!(output.usage.unwrap()["total_tokens"], 23);
        if original.is_some() {
            assert!(header.starts_with("POST /v1/images/edits "));
            assert!(header.contains("multipart/form-data"));
            assert_eq!(
                data.windows(png.len())
                    .filter(|w| *w == png.as_slice())
                    .count(),
                1
            );
            let text = String::from_utf8_lossy(&data);
            for (name, value) in [
                ("prompt", "Clay P icon"),
                ("model", MODEL.id),
                ("size", "1024x1024"),
                ("quality", "low"),
                ("n", "1"),
                ("output_format", "png"),
            ] {
                assert!(text.contains(&format!("name=\"{name}\"\r\n\r\n{value}\r\n")));
            }
            assert!(text.contains("name=\"image\"; filename=\"original.png\""));
        } else {
            assert!(header.starts_with("POST /v1/images/generations "));
            assert_eq!(
                serde_json::from_slice::<serde_json::Value>(&data).unwrap(),
                serde_json::json!({"model":MODEL.id,"prompt":"Clay P icon","size":"1024x1024","quality":"low","n":1,"output_format":"png"})
            );
        }
    }
}
#[test]
fn rejected_requests_produce_actionable_messages_without_echoing_server_secrets() {
    for (status, code, fragment) in [
        (401, "invalid_api_key", "Replace"),
        (403, "permission_denied", "access"),
        (429, "insufficient_quota", "billing"),
        (429, "rate_limit_exceeded", "Wait"),
        (500, "server_error", "unconfirmed"),
    ] {
        let (provider, server) = serve(
            status,
            serde_json::json!({"error":{"code":code,"message":"fictional-http-key"}}).to_string(),
        );
        let error = provider.render("icon", None).err().unwrap();
        server.join().unwrap();
        assert!(error.message.contains(fragment));
        assert!(!error.message.contains("fictional-http-key"));
        assert_eq!(error.uncertain, status == 500);
    }
}

#[test]
fn a_refused_connection_has_a_network_action_and_no_secret() {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let endpoint = format!("http://{}/v1/images", listener.local_addr().unwrap());
    drop(listener);
    let mut provider = OpenAi::new("fictional-network-key".into()).unwrap();
    provider.test_origin = Some(endpoint);
    provider.client = Client::builder().no_proxy().build().unwrap();
    let error = provider.render("icon", None).err().unwrap();
    assert!(error.message.contains("connection"));
    assert!(!error.message.contains("fictional-network-key"));
}
