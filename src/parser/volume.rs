use crate::{
    element::{Element, ElementKind},
    keyword::KeywordKind,
    tokenizer::Token,
    utils::*,
};

pub fn parse_volume<'a>(tokens: &mut [Token<'a>], results: &mut Vec<Element<'a>>) {
    for index in 0..tokens.len() {
        if !tokens[index]
            .keyword
            .is_some_and(|k| k.kind == KeywordKind::Volume)
        {
            continue;
        }

        let Some(next) = find_next_token(tokens, index, true, |t| t.is_not_delimiter()) else {
            continue;
        };
        if !tokens[next].is_free() {
            continue;
        }

        if super::episode::parse_multi_episode_range(tokens, next, results, ElementKind::Volume) {
            tokens[index].mark_known();
            tokens[next].mark_known();
            continue;
        }

        let Some((prefix, suffix)) = super::episode::parse_single_episode(tokens[next].value)
        else {
            continue;
        };
        results.push(Element {
            kind: ElementKind::Volume,
            value: prefix.into(),
            position: index,
        });
        if !suffix.is_empty() {
            results.push(Element {
                kind: ElementKind::ReleaseVersion,
                value: suffix.into(),
                position: index,
            })
        }
        tokens[index].mark_known();
        tokens[next].mark_known();
    }
}
