use crate::{
    element::{Element, ElementKind},
    keyword::KeywordKind,
    tokenizer::Token,
};
use std::borrow::Cow;

pub fn keyword_kind_to_element_kind(keyword: KeywordKind) -> Option<ElementKind> {
    match keyword {
        KeywordKind::AudioChannels => Some(ElementKind::AudioTerm),
        KeywordKind::AudioCodec => Some(ElementKind::AudioTerm),
        KeywordKind::AudioLanguage => Some(ElementKind::AudioTerm),
        KeywordKind::DeviceCompatibility => Some(ElementKind::DeviceCompatibility),
        KeywordKind::EpisodeType => Some(ElementKind::Type),
        KeywordKind::Language => Some(ElementKind::Language),
        KeywordKind::Other => Some(ElementKind::Other),
        KeywordKind::ReleaseGroup => Some(ElementKind::ReleaseGroup),
        KeywordKind::ReleaseInformation => Some(ElementKind::ReleaseInformation),
        KeywordKind::ReleaseVersion => Some(ElementKind::ReleaseVersion),
        KeywordKind::Source => Some(ElementKind::Source),
        KeywordKind::Subtitles => Some(ElementKind::Subtitles),
        KeywordKind::Type => Some(ElementKind::Type),
        KeywordKind::VideoCodec => Some(ElementKind::VideoTerm),
        KeywordKind::VideoColorDepth => Some(ElementKind::VideoTerm),
        KeywordKind::VideoFormat => Some(ElementKind::VideoTerm),
        KeywordKind::VideoFrameRate => Some(ElementKind::VideoTerm),
        KeywordKind::VideoProfile => Some(ElementKind::VideoTerm),
        KeywordKind::VideoQuality => Some(ElementKind::VideoTerm),
        KeywordKind::VideoResolution => Some(ElementKind::VideoResolution),
        _ => None,
    }
}

pub fn parse_keywords<'a>(
    tokens: &mut [Token<'a>],
    options: &crate::Options,
    results: &mut Vec<Element<'a>>,
) {
    for token in tokens.iter_mut().filter(|t| t.is_free()) {
        let Some(keyword) = token.keyword else {
            continue;
        };

        if keyword.kind == KeywordKind::ReleaseGroup && !options.parse_release_group() {
            continue;
        }
        if keyword.kind == KeywordKind::VideoResolution && !options.parse_video_resolution() {
            continue;
        }

        let Some(element_kind) = keyword_kind_to_element_kind(keyword.kind) else {
            continue;
        };

        if !keyword.is_ambiguous() || token.is_enclosed {
            token.mark_known();
        }

        let value = match keyword.kind {
            KeywordKind::ReleaseVersion => &token.value[1..],
            _ => token.value,
        };
        results.push(Element {
            kind: element_kind,
            value: Cow::Borrowed(value),
            position: token.position,
        });
    }
}
