use crate::{
    element::{Element, ElementKind},
    tokenizer::Token,
    utils::*,
};

pub fn parse_file_extension<'a>(tokens: &mut [Token<'a>]) -> Option<Element<'a>> {
    let [previous, last] = last_chunk_mut(tokens)?;
    let is_file_extension = last
        .keyword
        .is_some_and(|x| x.kind == crate::keyword::KeywordKind::FileExtension);
    let is_dot = previous.is_delimiter() && previous.value == ".";
    if is_file_extension && is_dot {
        previous.mark_known();
        last.mark_known();
        Some(Element::new(ElementKind::FileExtension, last))
    } else {
        None
    }
}
