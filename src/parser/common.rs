use crate::{tokenizer::Token, utils::*};

pub fn is_token_isolated(tokens: &[Token<'_>], index: usize) -> bool {
    let Some(previous) = find_prev_token(tokens, Some(index), |t| t.is_not_delimiter()) else {
        return false;
    };

    if !tokens[previous].is_bracket() {
        return false;
    }

    let Some(next) = find_next_token(tokens, index, true, |t| t.is_not_delimiter()) else {
        return false;
    };
    tokens[next].is_bracket()
}

pub fn is_valid_episode_number(s: &str) -> bool {
    !s.is_empty() && s.len() <= 4 && s.bytes().all(|x| x.is_ascii_digit())
}

pub fn is_japanese_number(ch: char) -> bool {
    matches!(
        ch,
        '〇' | '一' | '二' | '三' | '四' | '五' | '六' | '七' | '八' | '九' | '十' | '百' | '千'
    )
}

pub fn is_valid_japanese_episode(s: &str) -> bool {
    if s.is_ascii() {
        is_valid_episode_number(s)
    } else {
        let codepoints = s.chars().count();
        codepoints > 0 && codepoints <= 4 && s.chars().all(is_japanese_number)
    }
}
