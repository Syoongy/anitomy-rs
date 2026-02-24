use crate::{
    element::{Element, ElementKind},
    keyword::KeywordKind,
    tokenizer::Token,
    utils::*,
};

pub fn parse_part<'a>(tokens: &mut [Token<'a>], results: &mut Vec<Element<'a>>) {
    for index in 0..tokens.len() {
        if !tokens[index]
            .keyword
            .is_some_and(|k| k.kind == KeywordKind::Part)
        {
            continue;
        }

        let Some(next) = find_next_token(tokens, index, true, |t| t.is_not_delimiter()) else {
            continue;
        };
        if !tokens[next].is_number() {
            continue;
        }

        results.push(Element::new(ElementKind::Part, &tokens[next]));
        tokens[index].mark_known();
        tokens[next].mark_known();
    }
}
