use libc::{c_char, wchar_t};

use errno::errno;

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
        pub fn iswblank_native(ch: wint_t) -> libc::c_int;
        pub fn towupper_native(ch: wint_t) -> wint_t;
        pub fn towlower_native(ch: wint_t) -> wint_t;
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
    /// std::env::set_var("LC_ALL", "POSIX");
    /// assert!(!'\u{2003}'.is_space());
    /// std::env::set_var("LC_ALL", "en_US");
    /// assert!('\u{2003}'.is_space());
    /// ```
    fn is_space(&self) -> bool;

    /// Checks if `self` is classified as blank character (that is, a whitespace character used to separate words in a sentence) by the current locale.
    ///
    /// # examples
    ///
    /// ```
    /// use rust_locale::CType;
    ///
    /// assert!(' '.is_blank());
    /// assert!(!'\n'.is_blank());
    /// std::env::set_var("LC_ALL", "POSIX");
    /// assert!(!'\u{3000}'.is_blank());
    /// std::env::set_var("LC_ALL", "en_US");
    /// assert!('\u{3000}'.is_blank());
    /// ```
    fn is_blank(&self) -> bool;

    /// Converts `self` to uppercase listed in the current locale.
    ///
    /// If no uppercase version is listed in the current locale, returns unmodified `self`.
    ///
    /// Only 1:1 character mapping can be performed by this function, e.g. the uppercase form of '??' is (with some exceptions)
    /// the two-character string "SS", which cannot be obtained.
    ///
    /// # examples
    ///
    /// ```
    /// use rust_locale::CType;
    ///
    /// assert_eq!(CType::to_uppercase(&'a'), 'A');
    /// assert_eq!(CType::to_uppercase(&'1'), '1');
    /// std::env::set_var("LC_ALL", "POSIX");
    /// assert_eq!(CType::to_uppercase(&'\u{017F}'), '\u{017F}');
    /// std::env::set_var("LC_ALL", "en_US");
    /// assert_eq!(CType::to_uppercase(&'\u{017F}'), 'S');
    /// ```
    fn to_uppercase(&self) -> Self;

    /// Converts `self` to lowercase, if possible.
    ///
    /// If no lowercase version is listed in the current locale, returns unmodified `self`.
    ///
    /// Only 1:1 character mapping can be performed by this function, e.g. the Greek uppercase letter '??' has two lowercase forms,
    /// depending on the position in a word: '??' and '??'. A call to this method cannot be used to obtain the correct lowercase form in this case.
    ///
    /// # examples
    ///
    /// ```
    /// use rust_locale::CType;
    ///
    /// assert_eq!(CType::to_lowercase(&'A'), 'a');
    /// assert_eq!(CType::to_lowercase(&'1'), '1');
    /// std::env::set_var("LC_ALL", "POSIX");
    /// assert_eq!(CType::to_lowercase(&'\u{0190}'), '\u{0190}');
    /// std::env::set_var("LC_ALL", "en_US");
    /// assert_eq!(CType::to_lowercase(&'\u{0190}'), '\u{025b}');
    /// ```
    fn to_lowercase(&self) -> Self;
}

impl CType for char {
    fn is_space(&self) -> bool {
        let buf = utf8_bytes(self);
        if buf.len() == 1 {
            unsafe { libc::isspace(buf[0].into()) != 0 }
        } else {
            let wc = utf8towc(&buf);
            isspace(wc)
        }
    }

    fn is_blank(&self) -> bool {
        let buf = utf8_bytes(self);
        if buf.len() == 1 {
            unsafe { libc::isblank(buf[0].into()) != 0 }
        } else {
            let wc = utf8towc(&buf);
            isblank(wc)
        }
    }

    fn to_uppercase(&self) -> char {
        let bytes = utf8_bytes(self);
        let wc = utf8towc(&bytes);
        let upper = toupper(wc);
        wctochar(upper)
    }

    fn to_lowercase(&self) -> char {
        let bytes = utf8_bytes(self);
        let wc = utf8towc(&bytes);
        let lower = tolower(wc);
        wctochar(lower)
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
        s => panic!("utf8towc failed. status={}, error={}", s, errno()),
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
        status => panic!("wctochar failed. status={}, error={}", status, errno()),
    }
}

fn isspace(wc: wchar_t) -> bool {
    match unsafe { c::iswspace_native(wc.into()) } {
        s if s >= 0 => s != 0,
        _ => panic!("iswspace_native failed. error={}", errno()),
    }
}

fn isblank(wc: wchar_t) -> bool {
    unsafe { c::iswblank_native(wc.into()) != 0 }
}

fn toupper(wc: wchar_t) -> wchar_t {
    unsafe { c::towupper_native(wc.into()) as wchar_t }
}

fn tolower(wc: wchar_t) -> wchar_t {
    unsafe { c::towlower_native(wc.into()) as wchar_t }
}

#[cfg(test)]
mod tests {
    use super::*;

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
        std::env::set_var("LC_ALL", "POSIX");
        assert!(!'\u{1680}'.is_space());
        assert!(!'\u{2000}'.is_space());
        assert!(!'\u{2006}'.is_space());
        assert!(!'\u{2008}'.is_space());
        assert!(!'\u{200A}'.is_space());
        assert!(!'\u{2028}'.is_space());
        assert!(!'\u{2029}'.is_space());
        assert!(!'\u{205F}'.is_space());
        assert!(!'\u{3000}'.is_space());
        std::env::set_var("LC_ALL", "en_US");
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
    #[ignore]
    fn is_space_special() {
        std::env::set_var("LC_ALL", "en_US");
        assert!(!'\u{1361}'.is_space());
        std::env::set_var("LC_ALL", "am_ET");
        assert!('\u{1361}'.is_space());
    }

    #[test]
    fn is_blank() {
        std::env::set_var("LC_ALL", "POSIX");
        assert!(' '.is_blank());
        assert!('\t'.is_blank());
        assert!(!'\n'.is_blank());
        assert!(!'\u{3000}'.is_blank());
        std::env::set_var("LC_ALL", "en_US");
        assert!('\u{3000}'.is_blank());
        assert!(!'\u{2028}'.is_blank());
    }

    #[test]
    fn to_uppercase() {
        assert_eq!(CType::to_uppercase(&'a'), 'A');
        assert_eq!(CType::to_uppercase(&'1'), '1');
        std::env::set_var("LC_ALL", "POSIX");
        assert_eq!(CType::to_uppercase(&'\u{017F}'), '\u{017F}');
        std::env::set_var("LC_ALL", "en_US");
        assert_eq!(CType::to_uppercase(&'\u{017F}'), 'S');
    }

    #[test]
    #[ignore]
    fn to_uppercase_special() {
        std::env::set_var("LC_ALL", "en_US");
        assert_eq!(CType::to_uppercase(&'i'), 'I');
        std::env::set_var("LC_ALL", "tr_TR");
        assert_eq!(CType::to_uppercase(&'i'), '\u{0130}');
    }

    #[test]
    fn to_lowercase() {
        assert_eq!(CType::to_lowercase(&'A'), 'a');
        assert_eq!(CType::to_lowercase(&'1'), '1');
        std::env::set_var("LC_ALL", "POSIX");
        assert_eq!(CType::to_lowercase(&'\u{0190}'), '\u{0190}');
        std::env::set_var("LC_ALL", "en_US");
        assert_eq!(CType::to_lowercase(&'\u{0190}'), '\u{025b}');
    }

    #[test]
    #[ignore]
    fn to_lowercase_special() {
        std::env::set_var("LC_ALL", "en_US");
        assert_eq!(CType::to_lowercase(&'I'), 'i');
        std::env::set_var("LC_ALL", "tr_TR");
        assert_eq!(CType::to_lowercase(&'I'), '\u{0131}');
    }
}
