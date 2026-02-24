use crate::{
    element::{Element, ElementKind},
    tokenizer::Token,
};
use regex::Regex;
use std::sync::OnceLock;

fn is_video_resolution(input: &str) -> bool {
    static REGEX: OnceLock<Regex> = OnceLock::new();
    REGEX
        .get_or_init(|| Regex::new(r#"^\d{3,4}(?:[ipP]|[xX×]\d{3,4}[ipP]?)$"#).unwrap())
        .is_match(input)
}

pub fn parse_video_resolution<'a>(tokens: &mut [Token<'a>], results: &mut Vec<Element<'a>>) {
    let mut found = results
        .iter()
        .any(|e| e.kind == ElementKind::VideoResolution);
    for token in tokens
        .iter_mut()
        .filter(|t| t.is_free() && is_video_resolution(t.value))
    {
        token.mark_known();
        results.push(Element::new(ElementKind::VideoResolution, token));
        found = true;
    }

    if !found {
        if let Some(token) = tokens
            .iter_mut()
            .find(|t| t.is_free() && t.is_number() && (t.value == "1080" || t.value == "720"))
        {
            results.push(Element::new(ElementKind::VideoResolution, token));
        }
    }
}
