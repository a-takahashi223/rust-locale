use libc::{c_char, wchar_t};

use super::util::errno;

mod c {
    #[allow(non_camel_case_types)]
    type wint_t = i64;

    #[link(name = "rustlocale", kind = "static")]
    extern "C" {
        pub fn utf8towc(
            wc_buf: *mut libc::wchar_t,
            multibytes: *const libc::c_char,
            byte_length: libc::size_t,
        ) -> u8;
        pub fn wctoutf8(utf8_bytes: *mut libc::c_char, wc: libc::wchar_t) -> libc::ssize_t;
        pub fn iswspace_native(ch: wint_t) -> i8;
        pub fn towupper_native(ch: wint_t) -> wint_t;
    }
}

pub trait CType {
    /// Returns `true` if `self` is a whitespace character.
    ///
    /// Whitespace characters are:
    ///
    /// - space (0x20), form feed (0x0c), line feed (0x0a), carriage return (0x0d), horizontal tab (0x09), vertical tab (0x0b)
    /// - whitespace characters specific to the current locale
    ///
    /// # examples
    ///
    /// ```
    /// use rust_locale::CType;
    ///
    /// assert!(' '.is_space());
    /// assert!(!'a'.is_space());
    /// std::env::set_var("LANG", "POSIX");
    /// assert!(!'\u{2003}'.is_space());
    /// std::env::set_var("LANG", "en_US");
    /// assert!('\u{2003}'.is_space());
    /// ```
    fn is_space(&self) -> bool;

    /// Converts `self` to uppercase listed in the current locale.
    ///
    /// If no uppercase version is listed in the current locale, returns unmodified `self`.
    ///
    /// Only 1:1 character mapping can be performed by this function, e.g. the uppercase form of 'ß' is (with some exceptions)
    /// the two-character string "SS", which cannot be obtained.
    ///
    /// # examples
    ///
    /// ```
    /// use rust_locale::CType;
    ///
    /// assert_eq!(CType::to_uppercase(&'a'), 'A');
    /// assert_eq!(CType::to_uppercase(&'1'), '1');
    /// std::env::set_var("LANG", "POSIX");
    /// assert_eq!(CType::to_uppercase(&'\u{017F}'), '\u{017F}');
    /// std::env::set_var("LANG", "en_US");
    /// assert_eq!(CType::to_uppercase(&'\u{017F}'), 'S');
    /// assert_eq!(CType::to_uppercase(&'i'), 'I');
    /// std::env::set_var("LANG", "tr_TR");
    /// assert_eq!(CType::to_uppercase(&'i'), '\u{0130}');
    /// ```
    fn to_uppercase(&self) -> Self;
}

impl CType for char {
    fn is_space(&self) -> bool {
        let buf = utf8_bytes(self);
        if buf.len() == 1 {
            unsafe { libc::isspace(buf[0].into()) != 0 }
        } else {
            let wc = utf8towc(&buf);
            iswspace(wc)
        }
    }

    fn to_uppercase(&self) -> char {
        let bytes = utf8_bytes(self);
        let wc = utf8towc(&bytes);
        let upper = toupper(wc);
        wctochar(upper)
    }
}

fn utf8_bytes(c: &char) -> Vec<u8> {
    let length = c.len_utf8();
    let mut buf = vec![0; length];
    c.encode_utf8(&mut buf);
    buf
}

fn utf8towc(utf8_bytes: &Vec<u8>) -> wchar_t {
    let mut wc = 0;
    match unsafe {
        c::utf8towc(
            &mut wc as *mut wchar_t,
            utf8_bytes.as_ptr() as *const c_char,
            utf8_bytes.len(),
        )
    } {
        s if s == 0 => wc,
        s => panic!("utf8towc failed. status={}, errno={}", s, errno()),
    }
}

fn wctochar(wc: wchar_t) -> char {
    let mut buf = [0; 4];
    match unsafe { c::wctoutf8(buf.as_mut_ptr(), wc) } {
        length if length > 0 => {
            let length = length as usize;
            String::from_utf8(buf[..length].iter().map(|c| *c as u8).collect())
                .unwrap()
                .chars()
                .next()
                .unwrap()
        }
        status => panic!("wctochar failed. status={}, errno={}", status, errno()),
    }
}

fn iswspace(wc: wchar_t) -> bool {
    match unsafe { c::iswspace_native(wc.into()) } {
        s if s >= 0 => s != 0,
        _ => panic!("iswspace_native failed. errno={}", errno()),
    }
}

fn toupper(wc: wchar_t) -> wchar_t {
    unsafe { c::towupper_native(wc.into()) as wchar_t }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env as environ;

    #[test]
    fn is_always_space() {
        assert!(' '.is_space());
        assert!('\x0c'.is_space());
        assert!('\n'.is_space());
        assert!('\r'.is_space());
        assert!('\t'.is_space());
        assert!('\x0b'.is_space());
    }

    #[test]
    fn is_space_i18n() {
        environ::set_var("LANG", "POSIX");
        assert!(!'\u{1680}'.is_space());
        assert!(!'\u{2000}'.is_space());
        assert!(!'\u{2006}'.is_space());
        assert!(!'\u{2008}'.is_space());
        assert!(!'\u{200A}'.is_space());
        assert!(!'\u{2028}'.is_space());
        assert!(!'\u{2029}'.is_space());
        assert!(!'\u{205F}'.is_space());
        assert!(!'\u{3000}'.is_space());
        environ::set_var("LANG", "en_US");
        assert!('\u{1680}'.is_space());
        assert!('\u{2000}'.is_space());
        assert!('\u{2006}'.is_space());
        assert!('\u{2008}'.is_space());
        assert!('\u{200A}'.is_space());
        assert!('\u{2028}'.is_space());
        assert!('\u{2029}'.is_space());
        assert!('\u{205F}'.is_space());
        assert!('\u{3000}'.is_space());
    }

    #[test]
    fn is_space_special() {
        environ::set_var("LANG", "en_US");
        assert!(!'\u{1361}'.is_space());
        environ::set_var("LANG", "am_ET");
        assert!('\u{1361}'.is_space());
    }

    #[test]
    fn to_uppercase() {
        assert_eq!(CType::to_uppercase(&'a'), 'A');
        assert_eq!(CType::to_uppercase(&'1'), '1');
        std::env::set_var("LANG", "POSIX");
        assert_eq!(CType::to_uppercase(&'\u{017F}'), '\u{017F}');
        std::env::set_var("LANG", "en_US");
        assert_eq!(CType::to_uppercase(&'\u{017F}'), 'S');
        assert_eq!(CType::to_uppercase(&'i'), 'I');
        std::env::set_var("LANG", "tr_TR");
        assert_eq!(CType::to_uppercase(&'i'), '\u{0130}');
    }
}
