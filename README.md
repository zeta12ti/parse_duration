# parse_duration
[![Crates.io](https://img.shields.io/crates/v/parse_duration.svg)](https://crates.io/crates/parse_duration)
[![Travis](https://img.shields.io/travis/zeta12ti/parse_duration.svg)](https://travis-ci.org/zeta12ti/parse_duration)

This crate provides a function `parse` for parsing strings into durations.
The parser is based on the standard set by
[systemd.time](https://www.freedesktop.org/software/systemd/man/systemd.time.html#Parsing%20Time%20Spans),
but extends it significantly.
For example, negative numbers, decimals and exponents are allowed.

```
extern crate parse_duration;

use parse_duration::parse;
use std::time::Duration;

// One hour less than a day
assert_eq!(parse("1 day -1 hour").unwrap(), Duration::new(82_800, 0));
// Using exponents
assert_eq!(parse("1.26e-1 days").unwrap(), Duration::new(10_886, 400_000_000));
// Extra things will be ignored
assert_eq!(
    parse("Duration: 1 hour, 15 minutes and 29 seconds").unwrap(),
    Duration::new(4529, 0)
);
```


## Documentation
Documentation may be found [on docs.rs](https://docs.rs/parse_duration).


## Minimum Rust version policy
This crate's minimum supported rustc version is 1.28.0.

The current tentative policy is that the minimum Rust version required to use this crate can be increased in minor version updates. For example, if regex 1.0 requires Rust 1.20.0, then regex 1.0.z for all values of z will also require Rust 1.20.0 or newer. However, regex 1.y for y > 0 may require a newer minimum version of Rust.

In general, this crate will be conservative with respect to the minimum supported version of Rust.


## License
This software is licensed under the MIT License.


## Contributing
Feel free to file an issue or submit a pull request if there's a bug you want fixed
or a feature you want implemented.

By contributing to this project, you agree to license your code under the terms of
the MIT License.
