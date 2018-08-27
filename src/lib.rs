extern crate term_cursor;

use term_cursor::{get_pos, set_pos};

/// Try to read the preferred encoding from the locale.
///
/// The current implementation reads the POSIX environment variable `$LANG` and
/// extracts the codeset part of it. If it is "UTF-8" or "utf8", true is returned.
///
/// `$LANG` is assumed to follow XGP syntax: `language[_territory[.codeset]][@modifier]` <br>
/// The codeset part is taken to be everything after
/// the first '.' and before the first '@' (if it exists)
pub fn locale_requests_utf8() -> bool {
    match std::env::var("LANG") {
        Err(_) => false,
        Ok(lang) => {
            // either a path or invalid
            if lang.contains('/') {
                return false
            }
            // get the text between the first '.' and the first '@' (if it exists)
            // that should be the codeset
            let rest = match lang.split('.').nth(1) {
                Some(rest) => rest,
                None => return false,
            };
            let codeset = rest.split('@').next().unwrap();
            codeset.eq_ignore_ascii_case("UTF-8") || codeset.eq_ignore_ascii_case("utf8")
        }
    }
}

/// Tests support for single width unicode characters by printing a 2 byte, 1 wide character
/// and comparing cursor positions before and after.
///
/// _Note_: Blocks until it can acquire a lock on stdout.
pub fn supports_single_width_chars() -> std::io::Result<bool> {
    // 2 bytes, 1 wide. e with acute
    // no support if cursor moved 0 or 2 positions
    try_print("é", 1)
}

/// Tests support for double width unicode characters by printing a 3 byte, 2 wide character
/// and comparing cursor positions before and after. <br>
/// This is required to correctly display Chinese, Japanese and Korean (CJK) scripts.
///
/// _Note_: Blocks until it can acquire a lock on stdout.
pub fn supports_double_width_chars() -> std::io::Result<bool> {
    // 3 bytes, 2 wide. Chinese "hao" (= good)
    // no support if cursor moved 0 or 3 positions
    // likely unicode support, but not wide character support if it moved 1
    try_print("好", 2)
}

macro_rules! try_custom {
    ($b:expr) => {
        match $b {
            Ok(x) => x,
            Err(term_cursor::Error::PlatformSpecific) => return Ok(false),
            Err(term_cursor::Error::Io(io_error)) => return Err(io_error),
        }
    }
}

fn try_print(ch: &str, width: i32) -> std::io::Result<bool> {
    let (x, y) = try_custom!( get_pos() );
    print!("{}", ch);
    let (x_after, _) = try_custom!( get_pos() );

    // overwrite the printout with as many spaces as `ch` has bytes
    // to deal with terminals printing garbage for every byte
    // then reset the cursor pos
    try_custom!( set_pos(x, y) );
    print!("{:width$}", ' ', width = ch.len());
    try_custom!( set_pos(x, y) );

    Ok(x_after - x == width)
}

