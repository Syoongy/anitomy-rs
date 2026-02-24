use crate::{
    element::{Element, ElementKind},
    tokenizer::Token,
};

pub fn parse_file_checksum<'a>(tokens: &mut [Token<'a>]) -> Option<Element<'a>> {
    let (position, token) = tokens.iter_mut().enumerate().rev().find(|(_, t)| {
        t.is_free() && t.value.len() == 8 && t.value.bytes().all(|b| b.is_ascii_hexdigit())
    })?;

    token.mark_known();
    Some(Element {
        kind: ElementKind::FileChecksum,
        value: token.value.into(),
        position,
    })
}
