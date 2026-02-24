pub mod common;
pub mod episode;
pub mod episode_title;
pub mod file_checksum;
pub mod file_extension;
pub mod keywords;
pub mod part;
pub mod release_group;
pub mod season;
pub mod title;
pub mod video_resolution;
pub mod volume;
pub mod year;

use crate::{
    element::{Element, ElementKind},
    tokenizer::Token,
    Options,
};

pub(crate) fn parse_with_options(mut tokens: Vec<Token<'_>>, options: Options) -> Vec<Element<'_>> {
    let mut results = Vec::new();
    if options.parse_file_extension() {
        if let Some(el) = file_extension::parse_file_extension(&mut tokens) {
            results.push(el);
        }
    }

    keywords::parse_keywords(&mut tokens, &options, &mut results);

    if options.parse_file_checksum() {
        if let Some(el) = file_checksum::parse_file_checksum(&mut tokens) {
            results.push(el);
        }
    }

    if options.parse_video_resolution() {
        video_resolution::parse_video_resolution(&mut tokens, &mut results);
    }

    if options.parse_date() {
        if let Some(el) = year::parse_date(&mut tokens) {
            results.push(el);
        }
    }

    if options.parse_year() {
        if let Some(el) = year::parse_year(&mut tokens) {
            results.push(el);
        }
    }

    if options.parse_season() {
        season::parse_season(&mut tokens, &mut results);
    }

    part::parse_part(&mut tokens, &mut results);

    if options.parse_episode() {
        volume::parse_volume(&mut tokens, &mut results);
        episode::parse_episode(&mut tokens, &mut results, ElementKind::Episode);
    }

    if options.parse_title() {
        if let Some(title) = title::parse_title(&mut tokens) {
            results.push(title);
        }
    }

    if options.parse_release_group() && !results.iter().any(|e| e.kind == ElementKind::ReleaseGroup)
    {
        if let Some(group) = release_group::parse_release_group(&mut tokens) {
            results.push(group);
        }
    }

    let has_episode = results.iter().any(|e| e.kind == ElementKind::Episode);

    if has_episode {
        if options.parse_episode_title() {
            if let Some(title) = episode_title::parse_episode_title(&mut tokens) {
                results.push(title);
            }
        }

        if options.parse_episode() {
            episode::parse_episode(&mut tokens, &mut results, ElementKind::EpisodeAlt)
        }
    }

    results.sort_by_key(|e| e.position);
    results
}
