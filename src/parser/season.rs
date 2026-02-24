use crate::{
    element::{Element, ElementKind},
    keyword::KeywordKind,
    tokenizer::Token,
    utils::*,
};

fn inner_parse_season<'a>(tokens: &mut [Token<'a>]) -> Option<Element<'a>> {
    let is_season_keyword =
        |token: &Token<'a>| token.keyword.is_some_and(|x| x.kind == KeywordKind::Season);

    let mut iter = windows_mut(tokens);
    while let Some([first, mid, last]) = iter.next() {
        if is_season_keyword(last) && mid.is_delimiter() && first.is_free() {
            if let Some(number) = from_ordinal_number(first.value) {
                last.mark_known();
                mid.mark_known();
                first.mark_known();
                return Some(Element {
                    kind: ElementKind::Season,
                    value: number.into(),
                    position: first.position,
                });
            }
        }
        if is_season_keyword(first) && mid.is_delimiter() && last.is_free() {
            let value = if last.is_number() {
                last.value
            } else {
                match from_roman_number(last.value) {
                    Some(value) => value,
                    None => continue,
                }
            };
            last.mark_known();
            mid.mark_known();
            first.mark_known();
            return Some(Element {
                kind: ElementKind::Season,
                value: value.into(),
                position: last.position,
            });
        }
    }
    None
}

pub fn parse_season<'a>(tokens: &mut [Token<'a>], results: &mut Vec<Element<'a>>) {
    if let Some(result) = inner_parse_season(tokens) {
        results.push(result);
        return;
    }

    for token in tokens.iter_mut().filter(|x| x.is_free()) {
        if let Some(suffix) = token.value.strip_prefix(['S', 's']) {
            if let Some((first, second)) = token.value.split_once(['-', '~', '&', '+']) {
                let first_suffix: Option<&str> = first.strip_prefix(['S', 's']);
                let second_suffix: &str = second.strip_prefix(['S', 's']).unwrap_or(second);

                if let (Some(f), s) = (first_suffix, second_suffix) {
                    if (1..=2).contains(&f.len())
                        && f.bytes().all(|x: u8| x.is_ascii_digit())
                        && (1..=2).contains(&s.len())
                        && s.bytes().all(|x: u8| x.is_ascii_digit())
                    {
                        token.mark_known();
                        results.push(Element {
                            kind: ElementKind::Season,
                            value: f.into(),
                            position: token.position,
                        });
                        results.push(Element {
                            kind: ElementKind::Season,
                            value: s.into(),
                            position: token.position,
                        });
                        continue;
                    }
                }
            }

            if (1..=2).contains(&suffix.len()) && suffix.bytes().all(|x| x.is_ascii_digit()) {
                token.mark_known();
                results.push(Element {
                    kind: ElementKind::Season,
                    value: suffix.into(),
                    position: token.position,
                });
                continue;
            }
        }

        for value in token.value.split(['.', '-', '&', '+', '~']) {
            if let Some(prefix) = value.strip_suffix('期') {
                let prefix = prefix.strip_prefix('第').unwrap_or(prefix);
                if (1..=2).contains(&prefix.len()) && prefix.bytes().all(|x| x.is_ascii_digit()) {
                    token.mark_known();
                    results.push(Element {
                        kind: ElementKind::Season,
                        value: prefix.into(),
                        position: token.position,
                    });
                }
            }
        }
    }
}
