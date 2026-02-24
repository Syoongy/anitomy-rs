use crate::{
    element::{Element, ElementKind},
    tokenizer::Token,
    utils::*,
};

fn get_last_index_for_release_group(tokens: &[Token<'_>], first: Option<usize>) -> Option<usize> {
    let other_bracket = find_prev_token(tokens, first, |t| !t.is_enclosed && t.is_open_bracket())
        .and_then(|i| {
            tokens[i]
                .value
                .chars()
                .next()
                .and_then(crate::tokenizer::opposite_bracket)
        });

    first.and_then(|index| match other_bracket {
        Some(bracket) => find_next_token(tokens, index, true, |t| {
            t.is_closed_bracket() && t.value.starts_with(bracket)
        }),
        None => find_next_token(tokens, index, true, |t| t.is_closed_bracket()),
    })
}

pub fn find_release_group<'a, 'b>(tokens: &'b mut [Token<'a>]) -> Option<&'b mut [Token<'a>]> {
    let mut first = tokens
        .iter()
        .position(|t| t.is_enclosed && !t.is_identified());

    let mut last = get_last_index_for_release_group(tokens, first);

    while let Some((start, end)) = first.zip(last) {
        if start > tokens.len() || end > tokens.len() {
            break;
        }

        if tokens[start..end].iter().all(|t| !t.is_identified()) {
            break;
        }

        first = find_next_token(tokens, end, true, |t| t.is_enclosed && t.is_free());
        last = get_last_index_for_release_group(tokens, first);
    }

    if first.is_none() {
        if let Some(idx) = find_prev_token(tokens, Some(tokens.len()), |t| {
            t.is_free() && t.is_not_delimiter()
        }) {
            let token = &tokens[idx];
            if token.is_free()
                && idx != 0
                && tokens
                    .get(idx - 1)
                    .is_some_and(|t| t.is_delimiter() && t.value == "-")
            {
                first = Some(idx);
                last = Some(idx + 1);
            }
        }
    }

    match (first, last) {
        (Some(x), Some(y)) => Some(&mut tokens[x..y]),
        (Some(x), None) => Some(&mut tokens[x..]),
        _ => None,
    }
}

pub fn parse_release_group<'a>(tokens: &mut [Token<'a>]) -> Option<Element<'a>> {
    let range = find_release_group(tokens)?;
    let value = crate::tokenizer::combine_tokens(range, crate::tokenizer::KeepDelimiters::Yes);
    if value.is_empty() {
        None
    } else {
        let position = range.first()?.position;
        for token in range {
            token.mark_known();
        }
        Some(Element {
            kind: ElementKind::ReleaseGroup,
            value: value.into(),
            position,
        })
    }
}
