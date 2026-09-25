//! Replays `vectors/semantic/v0_to_v1.json`: v0 input bytes, the v1 objects they become.
//!
//! Outputs compare by meaning, not by bytes: both sides are parsed and deep-compared, with an
//! absent member equal to a `null` one. Nothing rehashes migrated JSON, so byte equality
//! would pin serializer details nobody depends on. Paths compare exactly, because the ids
//! and filenames in them are derived. A post `content` holding an envelope is spelled as an
//! object on the expected side and compared parsed. Media compares byte for byte.
#![cfg(feature = "migrator")]

use pubky_social_specs::migrate::{transform, Migrated, MigrationCtx};
use pubky_social_specs::traits::HashId;
use pubky_social_specs::{PubkyId, PubkySocialObject, PubkySocialTag};
use serde_json::{Map, Value};

fn corpus() -> Value {
    let path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/vectors/semantic/v0_to_v1.json"
    );
    serde_json::from_slice(&std::fs::read(path).expect("vectors")).expect("vectors json")
}

fn bytes_of(input: &Value) -> Vec<u8> {
    match (input.get("raw"), input.get("body")) {
        (Some(Value::String(raw)), None) => raw.as_bytes().to_vec(),
        (None, Some(body)) => serde_json::to_vec(body).unwrap(),
        _ => panic!("an input carries exactly one of raw and body: {input}"),
    }
}

fn context(corpus: &Value) -> MigrationCtx {
    let owner = PubkyId::try_from(corpus["owner"].as_str().unwrap()).unwrap();
    let mut ctx = MigrationCtx::new(owner);
    for file in corpus["files"].as_array().unwrap() {
        ctx.read_v0_file(file["tsid"].as_str().unwrap(), &bytes_of(file))
            .expect("a fixture File reads");
    }
    ctx
}

fn member<'a>(object: &'a Map<String, Value>, key: &str) -> &'a Value {
    object.get(key).unwrap_or(&Value::Null)
}

fn semantic_eq(actual: &Value, expected: &Value) -> bool {
    match (actual, expected) {
        (Value::Object(a), Value::Object(e)) => a
            .keys()
            .chain(e.keys())
            .all(|k| semantic_eq(member(a, k), member(e, k))),
        (Value::Array(a), Value::Array(e)) => {
            a.len() == e.len() && a.iter().zip(e).all(|(a, e)| semantic_eq(a, e))
        }
        // An envelope carried as a JSON string inside `content`
        (Value::String(a), Value::Object(_)) => {
            serde_json::from_str::<Value>(a).is_ok_and(|a| semantic_eq(&a, expected))
        }
        (a, e) => a == e,
    }
}

fn describe(migrated: &Migrated) -> Value {
    let writes: Vec<Value> = migrated
        .writes
        .iter()
        .map(
            |(path, bytes)| match serde_json::from_slice::<Value>(bytes) {
                Ok(body) => serde_json::json!({ "path": path, "body": body }),
                Err(_) => {
                    serde_json::json!({ "path": path, "raw": String::from_utf8_lossy(bytes) })
                }
            },
        )
        .collect();
    let dropped: Vec<String> = migrated.dropped.iter().map(|d| d.to_string()).collect();
    serde_json::json!({ "writes": writes, "dropped": dropped })
}

fn check(name: &str, got: &Result<Migrated, pubky_social_specs::migrate::Skip>, expected: &Value) {
    if let Some(skip) = expected.get("skip") {
        let got = got.as_ref().map(describe).map_err(|s| s.to_string());
        assert_eq!(got, Err(skip.as_str().unwrap().to_string()), "{name}");
        return;
    }
    let migrated = match got {
        Ok(m) => m,
        Err(skip) => panic!("{name}: skipped as {skip}"),
    };
    let actual = describe(migrated);
    let want_writes = expected["writes"].as_array().unwrap();
    assert_eq!(
        migrated.writes.len(),
        want_writes.len(),
        "{name}: got {actual:#}"
    );
    for ((path, bytes), want) in migrated.writes.iter().zip(want_writes) {
        assert_eq!(
            path,
            want["path"].as_str().unwrap(),
            "{name}: got {actual:#}"
        );
        match (want.get("raw"), want.get("body")) {
            (Some(raw), None) => {
                assert_eq!(bytes, raw.as_str().unwrap().as_bytes(), "{name}")
            }
            (None, Some(body)) => {
                let got: Value = serde_json::from_slice(bytes).unwrap();
                assert!(semantic_eq(&got, body), "{name}: got {got:#}");
            }
            _ => panic!("{name}: an expected write carries exactly one of raw and body"),
        }
    }
    let want_dropped = expected
        .get("dropped")
        .cloned()
        .unwrap_or(Value::Array(vec![]));
    assert_eq!(actual["dropped"], want_dropped, "{name}");
}

#[test]
fn every_vector_migrates_to_its_expected_objects() {
    let corpus = corpus();
    let ctx = context(&corpus);
    let vectors = corpus["vectors"].as_array().unwrap();
    assert!(vectors.len() >= 30, "the vectors lost rows");
    for vector in vectors {
        let name = vector["name"].as_str().unwrap();
        let input = &vector["input"];
        let got = transform(input["path"].as_str().unwrap(), &bytes_of(input), &ctx);
        check(name, &got, &vector["expected"]);
    }
}

#[test]
fn every_output_reads_back_through_the_v1_reader() {
    let corpus = corpus();
    let ctx = context(&corpus);
    let owner = corpus["owner"].as_str().unwrap();
    let mut written = 0;
    for vector in corpus["vectors"].as_array().unwrap() {
        let input = &vector["input"];
        let Ok(migrated) = transform(input["path"].as_str().unwrap(), &bytes_of(input), &ctx)
        else {
            continue;
        };
        for (path, bytes) in &migrated.writes {
            let uri = format!("pubky://{owner}/{path}");
            PubkySocialObject::from_uri(&uri, bytes)
                .unwrap_or_else(|e| panic!("{}: {uri}: {e}", vector["name"]));
            written += 1;
        }
    }
    assert!(written >= 20, "too few outputs were checked: {written}");
}

/// Each vector's kind names the transform its path reaches, so a row cannot test one
/// transform while claiming another.
#[test]
fn every_vector_kind_matches_its_path() {
    for vector in corpus()["vectors"].as_array().unwrap() {
        let path = vector["input"]["path"].as_str().unwrap();
        let segment = path
            .strip_prefix("pub/pubky.app/")
            .and_then(|rest| rest.split('/').next())
            .unwrap();
        let kind = match segment {
            "profile.json" => "user",
            "posts" => "post",
            "follows" => "follow",
            "mutes" => "mute",
            "bookmarks" => "bookmark",
            "tags" => "tag",
            "files" => "file",
            "blobs" => "blob",
            "feeds" => "feed",
            "last_read" => "last_read",
            other => panic!("unexpected segment {other}"),
        };
        assert_eq!(vector["kind"], kind, "{}", vector["name"]);
    }
}

// ---- chain composition ----

const V1: &str = "/social/v1/";
const V2: &str = "/social/v2/";

fn respell(value: &mut Value, from: &str, to: &str) {
    match value {
        Value::String(s) => *s = s.replace(from, to),
        Value::Array(items) => items.iter_mut().for_each(|v| respell(v, from, to)),
        Value::Object(map) => map.values_mut().for_each(|v| respell(v, from, to)),
        _ => {}
    }
}

/// A stand-in `1 -> 2` step. Like any real step it reads its input through the frozen reader
/// of its source epoch before it changes anything. It then respells every `social/v1`
/// reference the object carries to `social/v2`, re-derives the one id that depends on a
/// reference (the tag's, from the respelled target) and moves the path up one epoch.
fn v1_to_v2(owner: &str, path: &str, bytes: &[u8]) -> Result<(String, Vec<u8>), String> {
    let object = PubkySocialObject::from_uri(format!("pubky://{owner}/{path}"), bytes)?;
    let v2_path = path.replacen(V1, V2, 1);
    let mut value = match object {
        PubkySocialObject::File(file) => return Ok((v2_path, file.0)),
        _ => serde_json::from_slice::<Value>(bytes).map_err(|e| e.to_string())?,
    };
    respell(&mut value, V1, V2);
    let v2_path = match serde_json::from_value::<PubkySocialTag>(value.clone()) {
        Ok(tag) if path.starts_with("pub/social/v1/tags/") => {
            format!("pub/social/v2/tags/{}.json", tag.create_id())
        }
        _ => v2_path,
    };
    Ok((v2_path, serde_json::to_vec(&value).unwrap()))
}

/// Transforms compose by serialize-then-reparse: step one's output is serialized, step two
/// reads it back through the epoch-one reader and changes it, and the result means the same
/// as the change applied to step one's object directly. Respelled back, step two's bytes
/// still read through the epoch-one reader as step one's object. Checked on every vector
/// that writes.
#[test]
fn transforms_compose_by_serialize_then_reparse() {
    let corpus = corpus();
    let ctx = context(&corpus);
    let owner = corpus["owner"].as_str().unwrap();
    let (mut composed, mut rekeyed) = (0, 0);
    for vector in corpus["vectors"].as_array().unwrap() {
        let name = vector["name"].as_str().unwrap();
        let input = &vector["input"];
        let Ok(step_one) = transform(input["path"].as_str().unwrap(), &bytes_of(input), &ctx)
        else {
            continue;
        };
        for (path, bytes) in &step_one.writes {
            let (v2_path, v2_bytes) =
                v1_to_v2(owner, path, bytes).unwrap_or_else(|e| panic!("{name}: {e}"));
            assert!(
                v2_path.starts_with("pub/social/v2/") || v2_path.starts_with("priv/social/v2/")
            );
            let Ok(direct) = serde_json::from_slice::<Value>(bytes) else {
                assert_eq!(&v2_bytes, bytes, "{name}: media changed in the chain");
                composed += 1;
                continue;
            };
            let mut expected = direct.clone();
            respell(&mut expected, V1, V2);
            let mut reparsed: Value = serde_json::from_slice(&v2_bytes).unwrap();
            assert!(
                semantic_eq(&reparsed, &expected),
                "{name}: the reparse moved a value"
            );
            // A tag whose target moved has a new id, derived from the reparsed bytes
            if path.starts_with("pub/social/v1/tags/")
                && direct["uri"].as_str().unwrap().contains(V1)
            {
                assert_ne!(
                    v2_path,
                    path.replacen(V1, V2, 1),
                    "{name}: the tag id did not follow"
                );
                rekeyed += 1;
            }
            respell(&mut reparsed, V2, V1);
            let back = serde_json::to_vec(&reparsed).unwrap();
            PubkySocialObject::from_uri(format!("pubky://{owner}/{path}"), &back)
                .unwrap_or_else(|e| panic!("{name}: {e}"));
            assert!(
                semantic_eq(&reparsed, &direct),
                "{name}: the round trip moved a value"
            );
            composed += 1;
        }
    }
    assert!(composed >= 20, "too few outputs were composed: {composed}");
    assert!(rekeyed >= 2, "too few tags were rekeyed: {rekeyed}");
}
