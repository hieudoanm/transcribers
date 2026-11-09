use whisper::{Format, Segment, Transcript};

fn sample() -> Transcript {
    Transcript {
        language: Some("en".to_string()),
        duration_ms: 5_200,
        segments: vec![
            Segment {
                start_ms: 0,
                end_ms: 5_200,
                text: "Hello world".to_string(),
            },
            Segment {
                start_ms: 65_000,
                end_ms: 66_001,
                text: "Say \"hi\"\nnow".to_string(),
            },
        ],
    }
}

#[test]
fn format_parse() {
    assert_eq!(Format::parse("srt").unwrap(), Format::Srt);
    assert_eq!(Format::parse("TXT").unwrap(), Format::Txt);
    assert_eq!(Format::parse("text").unwrap(), Format::Txt);
    assert!(Format::parse("bogus").is_err());
}

#[test]
fn format_extension() {
    assert_eq!(Format::Srt.extension(), "srt");
    assert_eq!(Format::Json.extension(), "json");
}

#[test]
fn render_txt() {
    assert_eq!(
        whisper::render(Format::Txt, &sample()),
        "Hello world\nSay \"hi\"\nnow\n"
    );
}

#[test]
fn render_srt() {
    let out = whisper::render(Format::Srt, &sample());
    assert!(out.contains("1\n00:00:00,000 --> 00:00:05,200\nHello world\n\n"));
    assert!(out.contains("2\n00:01:05,000 --> 00:01:06,001\nSay \"hi\"\nnow\n\n"));
}

#[test]
fn render_vtt() {
    let out = whisper::render(Format::Vtt, &sample());
    assert!(out.starts_with("WEBVTT\n\n"));
    assert!(out.contains("00:01:05.000 --> 00:01:06.001"));
}

#[test]
fn render_json() {
    let out = whisper::render(Format::Json, &sample());
    assert!(out.contains("\"language\":\"en\""));
    assert!(out.contains("\"duration_ms\":5200"));
    assert!(out.contains("{\"start\":65000,\"end\":66001,\"text\":\"Say \\\"hi\\\"\\nnow\"}"));
}

#[test]
fn render_json_null_language() {
    let mut t = sample();
    t.language = None;
    assert!(whisper::render(Format::Json, &t).contains("\"language\":null"));
}
