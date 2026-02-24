use crate::{
    element::{Element, ElementKind},
    tokenizer::Token,
    utils::*,
};

fn is_year(s: &str) -> bool {
    s.parse::<u16>()
        .ok()
        .is_some_and(|x| (1950..=2050).contains(&x))
}

fn is_month(s: &str) -> bool {
    s.parse::<u8>()
        .ok()
        .is_some_and(|month| (1..=12).contains(&month))
}

fn is_day(s: &str) -> bool {
    s.parse::<u8>().ok().is_some_and(|x| (1..=31).contains(&x))
}

pub fn parse_year<'a>(tokens: &mut [Token<'a>]) -> Option<Element<'a>> {
    if let Some(token) = tokens
        .windows(3)
        .enumerate()
        .find(|(_, x)| {
            x[0].is_open_bracket()
                && x[2].is_closed_bracket()
                && x[1].is_free()
                && x[1].is_number()
                && is_year(x[1].value)
        })
        .map(|(offset, _)| offset + 1)
        .and_then(|idx| tokens.get_mut(idx))
    {
        token.mark_known();
        return Some(Element::new(ElementKind::Year, token));
    }

    for index in tokens
        .iter()
        .filter(|p| p.is_free() && p.is_number() && !p.is_enclosed && is_year(p.value))
        .map(|p| p.position)
    {
        if super::common::is_token_isolated(tokens, index) {
            tokens[index].mark_known();
            return Some(Element::new(ElementKind::Year, &tokens[index]));
        }
    }

    None
}

pub fn parse_date<'a>(tokens: &mut [Token<'a>]) -> Option<Element<'a>> {
    let mut iter = windows_mut(tokens);
    while let Some([year_month, delimiter, day]) = iter.next() {
        if !(delimiter.is_delimiter() && delimiter.value.starts_with(['.', '-']) && day.is_number())
        {
            continue;
        }

        let Some((year, month)) = year_month.value.split_once(['.', '-']) else {
            continue;
        };
        if !is_year(year) || !is_month(month) || !is_day(day.value) {
            continue;
        }

        year_month.mark_known();
        delimiter.mark_known();
        day.mark_known();
        return Some(Element {
            kind: ElementKind::Date,
            value: std::borrow::Cow::Owned(format!(
                "{}{}{}",
                year_month.value, delimiter.value, day.value
            )),
            position: year_month.position,
        });
    }

    None
}
