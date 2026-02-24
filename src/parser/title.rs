use crate::{
    element::{Element, ElementKind},
    tokenizer::{combine_tokens, opposite_bracket, Token},
    utils::*,
};

pub fn find_title<'a, 'b>(tokens: &'b mut [Token<'a>]) -> Option<&'b mut [Token<'a>]> {
    let mut first = tokens.iter().position(|t| t.is_free() && !t.is_enclosed);
    let mut last =
        first.and_then(|index| find_next_token(tokens, index, true, |t| t.is_identified()));
    if first.is_none() {
        if let Some((opposite, index)) =
            find_pair_mut(tokens, |t| t.is_closed_bracket(), |t| t.is_open_bracket()).and_then(
                |(_, open)| {
                    open.value
                        .chars()
                        .next()
                        .and_then(opposite_bracket)
                        .zip(Some(open.position))
                },
            )
        {
            first = find_next_token(tokens, index, false, |t| t.is_free());
            last = first.and_then(|idx| {
                find_next_token(tokens, idx, true, |t| {
                    t.is_bracket() && t.value.starts_with(opposite)
                })
            });
        }
    }

    let index = first?;

    let slice = &tokens[index..last.unwrap_or(tokens.len())];
    let (count, last_index) = slice
        .iter()
        .enumerate()
        .filter_map(|(index, token)| token.is_open_bracket().then_some(index))
        .fold((0, 0), |acc, x| (acc.0 + 1, x));
    if count != 0 {
        let closed_count = slice.iter().filter(|t| t.is_closed_bracket()).count();
        if closed_count != count {
            last = Some(last_index + index);
        }
    }

    if let Some(idx) = find_prev_token(tokens, last, |t| t.is_not_delimiter()) {
        let token = &tokens[idx];
        if token.is_closed_bracket() && token.value != ")" {
            if let Some(new_last) = find_prev_token(tokens, Some(idx), |t| t.is_open_bracket()) {
                last = Some(new_last)
            }
        }
    }

    match (first, last) {
        (Some(x), Some(y)) => Some(&mut tokens[x..y]),
        (Some(x), None) => Some(&mut tokens[x..]),
        _ => None,
    }
}

pub fn parse_title<'a>(tokens: &mut [Token<'a>]) -> Option<Element<'a>> {
    let range = find_title(tokens)?;
    let value = combine_tokens(range, crate::tokenizer::KeepDelimiters::No);
    if value.is_empty() {
        None
    } else {
        let position = range.first()?.position;
        for token in range {
            token.mark_known();
        }
        Some(Element {
            kind: ElementKind::Title,
            value: value.into(),
            position,
        })
    }
}
