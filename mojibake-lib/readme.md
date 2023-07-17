# mojibake

Encode and decode arbitrary bytes as a sequence of emoji optimized to produce the smallest number of graphemes.

## Description

This is not a space efficient library.

Generally services(Twitter,Mastodon,etc) will restrict the number of characters you're allowed to submit based on the grapheme count, not the literal character count. Singular emoji graphemes often consist of multi byte sequences that include multiple characters.

Therefore, if you can encode more data in a smaller number of graphemes, you can transmit more information while also having far more bytes than you otherwise would.

There are at least 2048 unique emoji graphemes in the unicode specification. Therefore an emoji is actually just an 11 bit unsigned integer with extra steps.

This library packs bytes bytes into 11 bit unsigned integers, which are then mapped to sequences of unicode characters that display as a single grapheme.

### Example

```raw
Original Text:
 Value: Shrek 2 was the greatest film ever made!!
 Bytes: 41,
 Characters: 41,
 Graphemes: 41

Mojibake Encoded:
 Value: 🇻🇳👌🏿🪀🔶🫳🏿🧏🏻📼🕺🏾🤛🏻🦺🤵🏽👦🏼🗄️💆🏿⚗️↗️2️⃣🧥🤵🏻🕤🙆🫚🪙😟🇦🇪🫳🏽🇸🇲😹🏴󠁧󠁢󠁳󠁣󠁴󠁿🛌🏻
 Bytes: 210,
 Characters: 55,
 Graphemes: 30

Decoded Text:
 Value: Shrek 2 was the greatest film ever made!!
 Bytes: 41,
 Characters: 41,
 Graphemes: 41
```
