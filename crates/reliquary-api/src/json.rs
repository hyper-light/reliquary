//! Manifests written as JSON (image spec v1.1.1, manifest.md and image-index.md): the bytes the
//! generator gives each manifest structure it makes (docs/design/model.md, H3). Fields come in one
//! order and absent ones are left out, so one structure has one encoding.

use crate::request::{Descriptor, ImageManifest, IndexManifest};

fn string(out: &mut String, text: &str) {
    out.push('"');
    for c in text.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            c if c.is_control() => out.push_str(&format!("\\u{:04x}", u32::from(c))),
            c => out.push(c),
        }
    }
    out.push('"');
}

fn field(out: &mut String, name: &str, value: &str) {
    out.push(',');
    string(out, name);
    out.push(':');
    string(out, value);
}

fn descriptor(out: &mut String, d: &Descriptor) {
    out.push_str("{\"mediaType\":");
    string(out, &d.media_type);
    field(out, "digest", &d.digest);
    out.push_str(&format!(",\"size\":{}}}", d.size));
}

fn descriptors(out: &mut String, name: &str, list: &[Descriptor]) {
    out.push(',');
    string(out, name);
    out.push_str(":[");
    for (i, d) in list.iter().enumerate() {
        if i > 0 {
            out.push(',');
        }
        descriptor(out, d);
    }
    out.push(']');
}

fn tail(out: &mut String, subject: Option<&Descriptor>, annotations: &[(String, String)]) {
    if let Some(subject) = subject {
        out.push_str(",\"subject\":");
        descriptor(out, subject);
    }
    if !annotations.is_empty() {
        out.push_str(",\"annotations\":{");
        for (i, (key, value)) in annotations.iter().enumerate() {
            if i > 0 {
                out.push(',');
            }
            string(out, key);
            out.push(':');
            string(out, value);
        }
        out.push('}');
    }
    out.push('}');
}

/// An image manifest's bytes.
pub(crate) fn image(m: &ImageManifest) -> Vec<u8> {
    let mut out = String::from("{\"schemaVersion\":2");
    if let Some(media_type) = &m.media_type {
        field(&mut out, "mediaType", media_type);
    }
    if let Some(artifact_type) = &m.artifact_type {
        field(&mut out, "artifactType", artifact_type);
    }
    out.push_str(",\"config\":");
    descriptor(&mut out, &m.config);
    descriptors(&mut out, "layers", &m.layers);
    tail(&mut out, m.subject.as_ref(), &m.annotations);
    out.into_bytes()
}

/// An index's bytes.
pub(crate) fn index(m: &IndexManifest) -> Vec<u8> {
    let mut out = String::from("{\"schemaVersion\":2");
    if let Some(media_type) = &m.media_type {
        field(&mut out, "mediaType", media_type);
    }
    if let Some(artifact_type) = &m.artifact_type {
        field(&mut out, "artifactType", artifact_type);
    }
    descriptors(&mut out, "manifests", &m.manifests);
    tail(&mut out, m.subject.as_ref(), &m.annotations);
    out.into_bytes()
}
