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
    let mut provider = OpenAi::new(
        "fictional-http-key".into(),
        crate::catalog::bundled().into_iter().next().unwrap(),
    )
    .unwrap();
    provider.client = Client::builder().no_proxy().build().unwrap();
    provider.test_origin = Some(endpoint);
    (provider, handle)
}
#[test]
fn sends_json_or_one_original_multipart_to_the_official_paths() {
    let png = include_bytes!("../../../icons/icon.png");
    use crate::{
        jobs::{self, Submission},
        library::Library,
        media::Format,
        providers::Fixture,
    };
    let dir = tempfile::tempdir().unwrap();
    let lib = Library::open(dir.path()).unwrap();
    let mut seed = Submission::new("JPEG original");
    seed.options.output_format = Format::Jpeg;
    jobs::submit(&lib, &Fixture::default(), &seed).unwrap();
    let source_id = lib.history(false).unwrap().remove(0).images.remove(0);
    let source = lib.original(&source_id).unwrap();
    let mut webp = std::io::Cursor::new(Vec::new());
    image::load_from_memory(png)
        .unwrap()
        .write_to(&mut webp, image::ImageFormat::WebP)
        .unwrap();
    let webp = webp.into_inner();
    for original in [None, Some(source_id.clone())] {
        let (provider,server)=serve(200,serde_json::json!({"data":vec![serde_json::json!({"b64_json":STANDARD.encode(&webp)});3],"usage":{"total_tokens":23}}).to_string());
        let mut draft = Submission::new("Clay P icon");
        draft.model = Some("gpt-image-2.6-flare".into());
        draft.input_id = original.clone();
        draft.options = ImageOptions {
            size: "1600x1024".into(),
            quality: "xhigh".into(),
            n: 3,
            output_format: Format::Webp,
            background: "transparent".into(),
        };
        let id = jobs::submit(&lib, &provider, &draft).unwrap();
        let job = lib
            .history(false)
            .unwrap()
            .into_iter()
            .find(|j| j.id == id)
            .unwrap();
        assert_eq!(job.model, "gpt-image-2.6-flare");
        assert_eq!(
            job.parameters,
            serde_json::to_value(&draft.options).unwrap()
        );
        assert_eq!(job.input_id, original);
        assert_eq!(job.images.len(), 3);
        for image in job.images {
            assert_eq!(lib.original(&image).unwrap().bytes, webp);
        }
        let (header, data) = server.join().unwrap();
        assert!(header
            .to_ascii_lowercase()
            .contains("authorization: bearer fictional-http-key"));
        assert_eq!(job.usage.unwrap()["total_tokens"], 23);
        if original.is_some() {
            assert!(header.starts_with("POST /v1/images/edits "));
            assert!(header.contains("multipart/form-data"));
            assert!(String::from_utf8_lossy(&data).contains("image/jpeg"));
            assert_eq!(
                data.windows(source.bytes.len())
                    .filter(|w| *w == source.bytes.as_slice())
                    .count(),
                1
            );
            let text = String::from_utf8_lossy(&data);
            for (name, value) in [
                ("prompt", "Clay P icon"),
                ("model", "gpt-image-2.6-flare"),
                ("size", "1600x1024"),
                ("quality", "xhigh"),
                ("n", "3"),
                ("background", "transparent"),
                ("output_format", "webp"),
            ] {
                assert!(text.contains(&format!("name=\"{name}\"\r\n\r\n{value}\r\n")));
            }
            assert!(text.contains("name=\"image\"; filename=\"original.jpeg\""));
        } else {
            assert!(header.starts_with("POST /v1/images/generations "));
            assert_eq!(
                serde_json::from_slice::<serde_json::Value>(&data).unwrap(),
                serde_json::json!({"model":"gpt-image-2.6-flare","prompt":"Clay P icon","size":"1600x1024","quality":"xhigh","n":3,"output_format":"webp","background":"transparent"})
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
        let error = provider
            .render("icon", None, &ImageOptions::default())
            .err()
            .unwrap();
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
    let mut provider = OpenAi::new(
        "fictional-network-key".into(),
        crate::catalog::bundled().into_iter().next().unwrap(),
    )
    .unwrap();
    provider.test_origin = Some(endpoint);
    provider.client = Client::builder().no_proxy().build().unwrap();
    let error = provider
        .render("icon", None, &ImageOptions::default())
        .err()
        .unwrap();
    assert!(error.message.contains("connection"));
    assert!(!error.message.contains("fictional-network-key"));
}
