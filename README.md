# rust-locale

[![CircleCI](https://circleci.com/gh/a-takahashi223/rust-locale/tree/main.svg?style=shield)](https://circleci.com/gh/a-takahashi223/rust-locale/tree/main)

`rust_locale` provides various functions dependent on locale specified in POSIX.1.

The main purpose is to provide something that is not in `char` methods or that differs in behavior from `char` methods.

## Dependency

```
[dependencies]
rust-locale = "0.1"
```

## Examples

```rust
use rust_locale::CType;

// space is different from whitespace
assert!('\x0c'.is_space());  # form feed
std::env::set_var("LC_ALL", "en_US");
assert!('\u{2003}'.is_space());  # Em Space
assert!(!'\u{1361}'.is_space());  # Ethiopic Wordspace
std::env::set_var("LC_ALL", "am_ET");
assert!('\u{1361}'.is_space());

// different behavior from char::to_uppercase
std::env::set_var("LC_ALL", "en_US");
assert_eq!(CType::to_uppercase(&'i'), 'I');
std::env::set_var("LC_ALL", "tr_TR");
assert_eq!(CType::to_uppercase(&'i'), '\u{0130}');  # Latin Capital Letter I with Dot Above
```

These tests may fail depending on the locales' definition.

## Future plan

- implement `to_lowercase`
- implement `is_blank`

Feature requests are welcome.

## License

`rust-locale` uses [Gnulib](https://www.gnu.org/software/gnulib/) for the platform compatibility. Gnulib is LGPL and `rust-locale` links to it statically, therefore `rust-locale` is also LGPL.

If you use `rust-locale` and not want to adapt LGPL, you may need to link to `rust-locale` dynamically.  
If that is difficult, I will consider linking `rust-locale` to Gnulib dynamically and change `rust-locale`'s license to MIT.
