#include <inttypes.h>
#include <locale.h>
#include <string.h>
#include <wchar.h>
#include <wctype.h>

/*
 * なぜか uselocale() を C の中でやらないといけない
 */

uint_fast8_t
utf8towc(wchar_t* wcbuf, const char* utf8_bytes, size_t length)
{
    const locale_t l = newlocale(LC_CTYPE_MASK, "C.UTF-8", 0);
    if (!l) {
        return 0x1;
    }
    (void)uselocale(l);

    uint_fast8_t ret = 0;

    mbstate_t state;
    (void)memset(&state, 0, sizeof state);
    if (mbrtowc(wcbuf, utf8_bytes, length, &state) != length) {
        ret = 0x2;
    }

    (void)uselocale(LC_GLOBAL_LOCALE);
    freelocale(l);
    return ret;
}

int_fast8_t
iswspace_native(wint_t ch)
{
    const locale_t l = newlocale(LC_CTYPE_MASK, "", 0);
    if (!l) {
        return -1;
    }
    (void)uselocale(l);

    const int ret = iswspace(ch);

    (void)uselocale(LC_GLOBAL_LOCALE);
    freelocale(l);
    return ret ? 1 : 0;
}