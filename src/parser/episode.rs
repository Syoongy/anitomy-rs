use crate::{
    element::{Element, ElementKind},
    keyword::KeywordKind,
    tokenizer::{is_dash, Token},
    utils::*,
};
use regex::Regex;
use std::sync::OnceLock;

fn episode_prefix_regex() -> &'static Regex {
    static REGEX: OnceLock<Regex> = OnceLock::new();
    REGEX
        .get_or_init(|| Regex::new(r#"^(?i:(?:E|EP|Eps)(\d{1,4}(?:\.5)?)(?:[vV](\d))?$)"#).unwrap())
}

fn season_and_episode_regex() -> &'static Regex {
    static REGEX: OnceLock<Regex> = OnceLock::new();
    REGEX.get_or_init(|| Regex::new(r#"^(?i:S?(\d{1,2})(?:-S?(\d{1,2}))?(?:x|[ ._-x]?EP?)(\d{1,4})(?:-(?:EP?)?(\d{1,4}))?(?:[vV](\d))?$)"#).unwrap())
}

fn number_sign_episode_regex() -> &'static Regex {
    static REGEX: OnceLock<Regex> = OnceLock::new();
    REGEX.get_or_init(|| Regex::new(r#"[#＃](\d{1,4})(?:[-~&+](\d{1,4}))?(?:[vV](\d))?"#).unwrap())
}

fn parse_number_in_number_episode<'a>(tokens: &mut [Token<'a>]) -> Option<Element<'a>> {
    for index in 0..tokens.len() {
        {
            let token = &tokens[index];
            if !(token.is_free() && token.is_number()) {
                continue;
            }
        }
        let Some(middle) = find_next_token(tokens, index, true, |t| {
            t.is_not_delimiter() || t.value == "&" || t.value == "~"
        }) else {
            continue;
        };
        let separator_value = tokens[middle].value;
        if separator_value != "&" && separator_value != "~" && separator_value != "of" {
            continue;
        }
        if index + 1 <= middle
            && tokens[index + 1..middle]
                .iter()
                .any(|t| t.is_not_delimiter())
        {
            continue;
        }
        if let Some(other_number) = tokens[middle..]
            .iter_mut()
            .skip(1)
            .find(|t| t.is_not_delimiter())
        {
            if !other_number.is_number() {
                continue;
            }
            if separator_value != "of" {
                other_number.mark_known();
            }
            tokens[middle].mark_known();
            tokens[index].mark_known();
            return Some(Element::new(ElementKind::Episode, &tokens[index]));
        }
    }
    None
}

pub fn parse_single_episode(s: &str) -> Option<(&str, &str)> {
    match s.split_once(['v', 'V']) {
        Some((prefix, suffix)) => {
            if super::common::is_valid_episode_number(prefix)
                && suffix.len() == 1
                && suffix.as_bytes()[0].is_ascii_digit()
            {
                Some((prefix, suffix))
            } else {
                None
            }
        }
        None if super::common::is_valid_episode_number(s) => Some((s, "")),
        _ => None,
    }
}

pub fn parse_multi_episode_range<'a>(
    tokens: &mut [Token<'a>],
    index: usize,
    results: &mut Vec<Element<'a>>,
    kind: ElementKind,
) -> bool {
    if let Some((first, last)) = tokens[index].value.split_once(['-', '~', '&', '+']) {
        let token = &mut tokens[index];
        if let Some(((lower, low_version), (upper, up_version))) =
            parse_single_episode(first).zip(parse_single_episode(last))
        {
            match lower.parse::<u16>().ok().zip(upper.parse::<u16>().ok()) {
                Some((x, y)) if x < y => {
                    results.push(Element {
                        kind,
                        value: lower.into(),
                        position: token.position,
                    });
                    token.mark_known();
                    if !low_version.is_empty() {
                        results.push(Element {
                            kind: ElementKind::ReleaseVersion,
                            value: low_version.into(),
                            position: token.position,
                        });
                    }
                    results.push(Element {
                        kind,
                        value: upper.into(),
                        position: token.position,
                    });
                    if !up_version.is_empty() {
                        results.push(Element {
                            kind: ElementKind::ReleaseVersion,
                            value: up_version.into(),
                            position: token.position,
                        });
                    }
                    return true;
                }
                _ => {}
            }
        }
    }
    false
}

pub fn parse_episode<'a>(
    tokens: &mut [Token<'a>],
    results: &mut Vec<Element<'a>>,
    kind: ElementKind,
) {
    let is_regular_episode = kind == ElementKind::Episode;

    for index in 0..tokens.len() {
        if !tokens[index].is_free() {
            continue;
        }

        let is_keyword = tokens[index]
            .keyword
            .is_some_and(|k| k.kind == KeywordKind::Episode);
        if is_keyword {
            if let Some(next) = find_next_token(tokens, index, true, |t| t.is_not_delimiter()) {
                if tokens[next].is_free() && tokens[next].is_mostly_numbers() {
                    if parse_multi_episode_range(tokens, next, results, kind) {
                        tokens[index].mark_known();
                        return;
                    }

                    if tokens[next].is_number() {
                        tokens[index].mark_known();
                        tokens[next].mark_known();
                        results.push(Element::new(kind, &tokens[next]));
                        return;
                    }
                }
            }
        }
    }

    for token in tokens.iter_mut().filter(|t| t.is_free()) {
        if let Some(m) = episode_prefix_regex().captures(token.value) {
            results.push(Element {
                kind,
                value: m.get(1).unwrap().as_str().into(),
                position: token.position,
            });
            token.mark_known();
            if let Some(inner) = m.get(2) {
                results.push(Element {
                    kind: ElementKind::ReleaseVersion,
                    value: inner.as_str().into(),
                    position: token.position,
                });
            }
            return;
        }
    }

    if let Some(number) = parse_number_in_number_episode(tokens) {
        results.push(number);
        return;
    }

    for token in tokens.iter_mut().filter(|t| t.is_free()) {
        if let Some((prefix, suffix)) = parse_single_episode(token.value) {
            if !suffix.is_empty() {
                token.mark_known();
                results.push(Element {
                    kind,
                    value: prefix.into(),
                    position: token.position,
                });
                results.push(Element {
                    kind: ElementKind::ReleaseVersion,
                    value: suffix.into(),
                    position: token.position,
                });
                return;
            }
        }
    }

    for index in 0..tokens.len() {
        if tokens[index].is_free() && parse_multi_episode_range(tokens, index, results, kind) {
            return;
        }
    }

    for token in tokens.iter_mut().filter(|t| t.is_free()) {
        if let Some(captures) = season_and_episode_regex().captures(token.value) {
            if captures[1].parse::<u8>().unwrap_or_default() != 0 {
                results.push(Element {
                    kind: ElementKind::Season,
                    value: captures.get(1).unwrap().as_str().into(),
                    position: token.position,
                });
                token.mark_known();
                if let Some(inner) = captures.get(2) {
                    results.push(Element {
                        kind: ElementKind::Season,
                        value: inner.as_str().into(),
                        position: token.position,
                    });
                }

                results.push(Element {
                    kind,
                    value: captures.get(3).unwrap().as_str().into(),
                    position: token.position,
                });
                if let Some(inner) = captures.get(4) {
                    results.push(Element {
                        kind,
                        value: inner.as_str().into(),
                        position: token.position,
                    });
                }
                if let Some(inner) = captures.get(5) {
                    results.push(Element {
                        kind: ElementKind::ReleaseVersion,
                        value: inner.as_str().into(),
                        position: token.position,
                    });
                }
                return;
            }
        }
    }

    if let Some((_, token)) = find_pair_mut(
        tokens,
        |t| {
            t.keyword.is_some_and(|x| x.kind == KeywordKind::Type)
                && !t.value.eq_ignore_ascii_case("movie")
        },
        |t| t.is_not_delimiter(),
    ) {
        if token.is_free() && token.is_number() {
            token.mark_known();
            results.push(Element::new(kind, token));
            return;
        }
    }

    for token in tokens.iter_mut().filter(|t| t.is_free()) {
        if let Some(captures) = number_sign_episode_regex().captures(token.value) {
            token.mark_known();
            results.push(Element {
                kind,
                value: captures.get(1).unwrap().as_str().into(),
                position: token.position,
            });
            if let Some(inner) = captures.get(2) {
                results.push(Element {
                    kind,
                    value: inner.as_str().into(),
                    position: token.position,
                });
            }
            if let Some(inner) = captures.get(3) {
                results.push(Element {
                    kind: ElementKind::ReleaseVersion,
                    value: inner.as_str().into(),
                    position: token.position,
                });
            }
            return;
        }
    }

    for token in tokens.iter_mut().filter(|t| t.is_free()) {
        if let Some(prefix) = token.value.strip_suffix('話') {
            let prefix = prefix.strip_prefix('第').unwrap_or(prefix);
            if super::common::is_valid_japanese_episode(prefix) {
                token.mark_known();
                results.push(Element {
                    kind,
                    value: prefix.into(),
                    position: token.position,
                });
                return;
            }
        }
    }

    if is_regular_episode {
        for index in tokens
            .iter()
            .filter(|t| t.is_free() && t.is_number())
            .map(|t| t.position)
        {
            if super::common::is_token_isolated(tokens, index)
                || !super::common::is_valid_episode_number(tokens[index].value)
            {
                continue;
            }

            let Some(next) = find_next_token(tokens, index, true, |t| t.is_not_delimiter()) else {
                continue;
            };
            if !tokens[next].is_bracket() {
                continue;
            }

            let Some(next) = find_next_token(tokens, next, true, |t| t.is_not_delimiter()) else {
                continue;
            };
            let is_valid = {
                let token = &tokens[next];
                token.is_free()
                    && token.is_number()
                    && super::common::is_valid_episode_number(token.value)
                    && super::common::is_token_isolated(tokens, next)
            };
            if !is_valid {
                continue;
            }

            let Some((first, second)) = tokens[index]
                .value
                .parse::<u16>()
                .ok()
                .zip(tokens[next].value.parse::<u16>().ok())
            else {
                continue;
            };

            let (a, b) = if first > second {
                (ElementKind::EpisodeAlt, ElementKind::Episode)
            } else {
                (ElementKind::Episode, ElementKind::EpisodeAlt)
            };

            tokens[next].mark_known();
            tokens[index].mark_known();
            results.push(Element::new(b, &tokens[next]));
            results.push(Element::new(a, &tokens[index]));
            return;
        }
    }

    for index in 0..tokens.len() {
        let is_valid = {
            let token = &tokens[index];
            token.is_delimiter() && token.value.chars().next().is_some_and(is_dash)
        };
        if !is_valid {
            continue;
        }

        if let Some(token) = tokens.iter_mut().skip(index).find(|x| x.is_not_delimiter()) {
            if token.is_number() && token.is_free() {
                token.mark_known();
                results.push(Element::new(kind, token));
                tokens[index].mark_known();
                return;
            }
        }
    }

    for token in tokens.iter_mut().filter(|t| t.is_free()) {
        if let Some((first, second)) = token.value.split_once('.') {
            if second == "5" && super::common::is_valid_episode_number(first) {
                token.mark_known();
                results.push(Element::new(kind, token));
                return;
            }
        }
    }

    {
        let mut iter = windows_mut(tokens);
        while let Some([first, middle, last]) = iter.next() {
            if first.is_open_bracket()
                && last.is_closed_bracket()
                && middle.is_free()
                && middle.is_number()
            {
                results.push(Element::new(kind, middle));
                middle.mark_known();
                return;
            }
        }
    }

    let partial_episode_indices: Vec<_> = tokens
        .iter()
        .enumerate()
        .filter(|(_, t)| t.is_free())
        .filter(|(_, t)| {
            if let Some(prefix) = t.value.strip_suffix(['A', 'B', 'C', 'a', 'b', 'c']) {
                super::common::is_valid_episode_number(prefix)
            } else {
                false
            }
        })
        .filter(|(i, t)| {
            if *i > 1 && t.value == "1a" {
                let prev_idx = i.saturating_sub(2);
                if prev_idx < tokens.len() && tokens[prev_idx].value == "Ver1" {
                    return false;
                }
            }
            true
        })
        .map(|(i, _)| i)
        .collect();

    for i in partial_episode_indices {
        let token = &mut tokens[i];
        token.mark_known();
        results.push(Element::new(kind, token));
        return;
    }

    for index in (0..tokens.len())
        .skip(1)
        .filter(|&i| tokens[i].is_free() && tokens[i].is_number() && !tokens[i].is_enclosed)
    {
        if tokens[..index]
            .iter()
            .all(|t| t.is_enclosed || t.is_delimiter())
        {
            continue;
        }

        let is_version_number = |idx: usize| -> bool {
            let prev = find_prev_token(tokens, Some(idx), |t| t.is_not_delimiter());
            if let Some(prev_idx) = prev {
                tokens[prev_idx].is_delimiter() && tokens[prev_idx].value == "."
            } else {
                false
            }
        };

        let previous = find_prev_token(tokens, Some(index), |t| t.is_not_delimiter());
        if let Some(idx) = previous {
            let prev = &tokens[idx];
            if prev.is_free()
                && (prev.value.eq_ignore_ascii_case("movie")
                    || prev.value.eq_ignore_ascii_case("part")
                    || prev.value.eq_ignore_ascii_case("cour")
                    || prev.value.eq_ignore_ascii_case("no"))
            {
                continue;
            }
            if is_version_number(idx) {
                continue;
            }
            if prev.value == "]" {
                continue;
            }
        }

        let next = find_next_token(tokens, index, true, |t| t.is_not_delimiter());
        if let Some(next_idx) = next {
            if is_version_number(next_idx) {
                continue;
            }
        }

        if let (Some(prev_idx), Some(next_idx)) = (previous, next) {
            if tokens[prev_idx].is_free() && tokens[next_idx].is_free() {
                continue;
            }
        }

        let token = &mut tokens[index];
        token.mark_known();
        results.push(Element::new(kind, token));
        break;
    }
}
