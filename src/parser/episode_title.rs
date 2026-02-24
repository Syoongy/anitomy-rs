use crate::{
    element::{Element, ElementKind},
    keyword::KeywordKind,
    tokenizer::{combine_tokens, KeepDelimiters, Token},
    utils::*,
};

pub fn find_episode_title<'a, 'b>(tokens: &'b mut [Token<'a>]) -> Option<&'b mut [Token<'a>]> {
    let mut first = tokens.iter().position(|t| t.is_free() && !t.is_enclosed);
    let mut last = first.and_then(|index| {
        find_next_token(tokens, index, false, |t| {
            t.is_open_bracket()
                || (t.is_identified() && t.keyword.map_or(false, |k| k.kind != KeywordKind::Part))
        })
    });

    if first.is_none() {
        first = tokens
            .iter()
            .position(|t| t.is_open_bracket() && t.value == "「")
            .map(|idx| idx + 1);
        last = first.and_then(|index| {
            find_next_token(tokens, index, false, |t| {
                t.is_closed_bracket() && t.value == "」"
            })
        });
        match last {
            None => return None,
            Some(last) => {
                if tokens[first.unwrap_or_default()..last]
                    .iter()
                    .any(|t| t.is_identified())
                {
                    return None;
                }
            }
        }
    }

    match (first, last) {
        (Some(x), Some(y)) => Some(&mut tokens[x..y]),
        (Some(x), None) => Some(&mut tokens[x..]),
        _ => None,
    }
}

pub fn parse_episode_title<'a>(tokens: &mut [Token<'a>]) -> Option<Element<'a>> {
    let range = find_episode_title(tokens)?;
    let value = combine_tokens(range, KeepDelimiters::No);
    if value.is_empty() {
        None
    } else {
        let position = range.first()?.position;
        for token in range {
            token.mark_known();
        }
        Some(Element {
            kind: ElementKind::EpisodeTitle,
            value: value.into(),
            position,
        })
    }
}
