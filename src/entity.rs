use ctype::isdigit;

use entities::ENTITIES;
use std::borrow::Cow;
use std::char;
use std::cmp::min;
use std::str;

pub const ENTITY_MIN_LENGTH: usize = 2;
pub const ENTITY_MAX_LENGTH: usize = 31;

fn isxdigit(ch: &u8) -> bool {
    (*ch >= b'0' && *ch <= b'9') || (*ch >= b'a' && *ch <= b'f') || (*ch >= b'A' && *ch <= b'F')
}

pub fn unescape(text: &[u8]) -> Option<(Cow<'static, [u8]>, usize)> {
    if text.len() >= 3 && text[0] == b'#' {
        let mut codepoint: u32 = 0;
        let mut i = 0;

        let num_digits = if isdigit(text[1]) {
            i = 1;
            while i < text.len() && isdigit(text[i]) {
                codepoint = (codepoint * 10) + (text[i] as u32 - '0' as u32);
                codepoint = min(codepoint, 0x11_0000);
                i += 1;
            }
            i - 1
        } else if text[1] == b'x' || text[1] == b'X' {
            i = 2;
            while i < text.len() && isxdigit(&text[i]) {
                codepoint = (codepoint * 16) + ((text[i] as u32 | 32) % 39 - 9);
                codepoint = min(codepoint, 0x11_0000);
                i += 1;
            }
            i - 2
        } else {
            0
        };

        if num_digits >= 1 && num_digits <= 8 && i < text.len() && text[i] == b';' {
            if codepoint == 0 || (codepoint >= 0xD800 && codepoint <= 0xE000) ||
                codepoint >= 0x110000
            {
                codepoint = 0xFFFD;
            }
            return Some((
                char::from_u32(codepoint).unwrap_or('\u{FFFD}').to_string().into_bytes().into(),
                i + 1,
            ));
        }
    }

    let size = min(text.len(), ENTITY_MAX_LENGTH);
    for i in ENTITY_MIN_LENGTH..size {
        if text[i] == b' ' {
            return None;
        }

        if text[i] == b';' {
            return lookup(&text[..i]).map(|e| (e.into(), i + 1));
        }
    }

    None
}

fn lookup(text: &[u8]) -> Option<&'static [u8]> {
    let entity_str = format!("&{};", unsafe {str::from_utf8_unchecked(text) });

    let entity = ENTITIES.iter().find(|e| e.entity == entity_str);

    match entity {
        Some(e) => Some(e.characters.as_bytes()),
        None => None,
    }
}

pub fn unescape_html<'a>(src: &Cow<'a, [u8]>) -> Cow<'a, [u8]> {
    let size = src.len();
    let mut i = 0;
    let mut v = vec![];

    while i < size {
        let org = i;
        while i < size && src[i] != b'&' {
            i += 1;
        }

        if i > org {
            if org == 0 && i >= size {
                return src.clone();
            }

            v.extend_from_slice(&src[org..i]);
        }

        if i >= size {
            return Cow::from(v);
        }

        i += 1;
        match unescape(&src[i..]) {
            Some((chs, size)) => {
                v.extend_from_slice(&chs);
                i += size;
            }
            None => v.push(b'&'),
        }
    }

    Cow::from(v)
}
