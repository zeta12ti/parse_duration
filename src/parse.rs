// MIT License
//
// Copyright (c) 2017 zeta12ti
//
// Permission is hereby granted, free of charge, to any person obtaining a copy
// of this software and associated documentation files (the "Software"), to deal
// in the Software without restriction, including without limitation the rights
// to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
// copies of the Software, and to permit persons to whom the Software is
// furnished to do so, subject to the following conditions:
//
// The above copyright notice and this permission notice shall be included in all
// copies or substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
// OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
// SOFTWARE.

use regex::Regex;
use std::time::Duration;
use std::fmt;
use std::error::Error;
use num::{BigInt, ToPrimitive};
use num::pow::pow;


#[derive(Debug, PartialEq, Eq, Clone)]
/// An enumeration of the possible errors while parsing.
pub enum ParseError {
    /// A string failed to be parsed as an integer.
    ParseIntError(String),
    /// An unrecognized unit was found.
    UnknownUnitError(String),
    /// A BigInt was too big to be converted into a u64 or was negative.
    OutOfBoundsError(BigInt),
    /// A value without a unit was found.
    NoUnitFoundError(String),
    /// No Value at all was found.
    NoValueFoundError(String),
}

// bring all the variants into scope
use ParseError::{ParseIntError, UnknownUnitError, OutOfBoundsError, NoUnitFoundError,
                 NoValueFoundError};

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            ParseIntError(ref s) => {
                write!(f, "ParseIntError: Failed to parse \"{}\" as an integer", s)
            }
            UnknownUnitError(ref s) => write!(f, "UnknownUnitError: \"{}\" is not a known unit", s),
            OutOfBoundsError(ref b) => {
                write!(f, "OutOfBoundsError: \"{}\" cannot be converted to u64", b)
            }
            NoUnitFoundError(ref s) => {
                write!(f, "NoUnitFoundError: no unit found for the value \"{}\"", s)
            }
            NoValueFoundError(ref s) => {
                write!(
                    f,
                    "NoValueFoundError: no value found in the string \"{}\"",
                    s
                )
            }
        }
    }
}

impl Error for ParseError {
    fn description(&self) -> &str {
        match *self {
            ParseIntError(_) => "Failed to parse a string into an integer",
            UnknownUnitError(_) => "An unknown unit was used",
            OutOfBoundsError(_) => "An integer was too large to convert into a u64",
            NoUnitFoundError(_) => "A value without a unit was found",
            NoValueFoundError(_) => "No value was found",
        }
    }
}

#[derive(Default)]
struct ProtoDuration {
    nanoseconds: BigInt,
    microseconds: BigInt,
    milliseconds: BigInt,
    seconds: BigInt,
    minutes: BigInt,
    hours: BigInt,
    days: BigInt,
    weeks: BigInt,
    months: BigInt,
    years: BigInt,
}

impl ProtoDuration {
    fn into_duration(self) -> Result<Duration, ParseError> {
        let mut nanoseconds = self.nanoseconds + 1_000_u32 * self.microseconds +
            1_000_000_u32 * self.milliseconds;
        let mut seconds = self.seconds + 60_u32 * self.minutes + 3_600_u32 * self.hours +
            86_400_u32 * self.days + 604_800_u32 * self.weeks +
            2_629_746_u32 * self.months + 31_556_952_u32 * self.years;

        seconds = seconds + (&nanoseconds / 1_000_000_000_u32);
        nanoseconds = nanoseconds % 1_000_000_000_u32;

        let seconds = <BigInt as ToPrimitive>::to_u64(&seconds).ok_or_else(|| {
            OutOfBoundsError(seconds)
        })?;
        let nanoseconds = <BigInt as ToPrimitive>::to_u32(&nanoseconds).ok_or_else(
            || {
                // This shouldn't happen since nanoseconds is less than 1 billion.
                OutOfBoundsError(nanoseconds)
            },
        )?;

        Ok(Duration::new(seconds, nanoseconds))
    }
}

lazy_static! {
    static ref NUMBER_RE: Regex = Regex::new(
        r"(?x)
        ^
        [^\w-]*     # any non-word characters, except '-' (for negatives - may add '.' for decimals)
        (-?\d+)     # a possible negative sign and some positive number of digits
        [^\w-]*     # more non-word characters
        $").expect("Compiling a regex went wrong");
}

lazy_static! {
    static ref DURATION_RE: Regex = Regex::new(
        r"(?x)(?i)
        (?P<int>-?\d+)              # the integer part
        \.?(?:(?P<dec>\d+))?        # an optional decimal part
                                    # note: the previous part will eat any decimals
                                    # if there's no decimal point.
                                    # This means we'll always have the decimal point if this
                                    # section matches at all.
        (?:e(?P<exp>[-+]?\d+))?     # an optional exponent
        (?:
            [^\w]*                  # some amount of junk (non word characters)
            (?P<unit>[\w&&[^\d]]+)  # a word with no digits
        )?
        ",
    ).expect("Compiling a regex went wrong");
}

fn parse_unit(unit: &str) -> &str {
    let unit_casefold = unit.to_lowercase();

    if unit_casefold.starts_with('n') &&
        ("nanoseconds".starts_with(&unit_casefold) || "nsecs".starts_with(&unit_casefold))
    {
        "nanoseconds"
    } else if unit_casefold.starts_with("mic") && "microseconds".starts_with(&unit_casefold) ||
               unit_casefold.starts_with('u') && "usecs".starts_with(&unit_casefold) ||
               unit_casefold.starts_with('μ') && "μsecs".starts_with(&unit_casefold)
    {
        "microseconds"
    } else if unit_casefold.starts_with("mil") && "milliseconds".starts_with(&unit_casefold) ||
               unit_casefold.starts_with("ms") && "msecs".starts_with(&unit_casefold)
    {
        "milliseconds"
    } else if unit_casefold.starts_with('s') &&
               ("seconds".starts_with(&unit_casefold) || "secs".starts_with(&unit_casefold))
    {
        "seconds"
    } else if (unit_casefold.starts_with("min") || unit.starts_with('m')) &&
               ("minutes".starts_with(&unit_casefold) || "mins".starts_with(&unit_casefold))
    {
        "minutes"
    } else if unit_casefold.starts_with('h') &&
               ("hours".starts_with(&unit_casefold) || "hrs".starts_with(&unit_casefold))
    {
        "hours"
    } else if unit_casefold.starts_with('d') && "days".starts_with(&unit_casefold) {
        "days"
    } else if unit_casefold.starts_with('w') && "weeks".starts_with(&unit_casefold) {
        "weeks"
    } else if (unit_casefold.starts_with("mo") || unit.starts_with('M')) &&
               "months".starts_with(&unit_casefold)
    {
        "months"
    } else if unit_casefold.starts_with('y') &&
               ("years".starts_with(&unit_casefold) || "yrs".starts_with(&unit_casefold))
    {
        "years"
    } else {
        unit
    }
}

/// Parse a string into a duration object.
///
/// See the [module level documentation](index.html) for more.
pub fn parse(input: &str) -> Result<Duration, ParseError> {
    if let Some(int) = NUMBER_RE.captures(input) {
        // This means it's just a value
        let seconds = BigInt::parse_bytes(int[1].as_bytes(), 10).ok_or_else(|| {
            ParseIntError(int[1].to_owned())
        })?;
        return Ok(Duration::new(
            seconds.to_u64().ok_or_else(|| OutOfBoundsError(seconds))?,
            0,
        ));
    } else if DURATION_RE.is_match(input) {
        // This means we have at least one "unit" (or plain word) and one value.
        let mut duration = ProtoDuration::default();
        for capture in DURATION_RE.captures_iter(input) {
            match (
                capture.name("int"),
                capture.name("dec"),
                capture.name("exp"),
                capture.name("unit"),
            ) {
                // capture.get(0) is *always* the actual match, so unwrapping causes no problems
                (.., None) => {
                    return Err(NoUnitFoundError(
                        capture.get(0).unwrap().as_str().to_owned(),
                    ))
                }
                (None, ..) => {
                    return Err(NoValueFoundError(
                        capture.get(0).unwrap().as_str().to_owned(),
                    ))
                }
                (Some(int), None, None, Some(unit)) => {
                    let int = BigInt::parse_bytes(int.as_str().as_bytes(), 10)
                        .ok_or_else(|| ParseIntError(int.as_str().to_owned()))?;

                    match parse_unit(unit.as_str()) {
                        "nanoseconds" => duration.nanoseconds = duration.nanoseconds + int,
                        "microseconds" => duration.microseconds = duration.microseconds + int,
                        "milliseconds" => duration.milliseconds = duration.milliseconds + int,
                        "seconds" => duration.seconds = duration.seconds + int,
                        "minutes" => duration.minutes = duration.minutes + int,
                        "hours" => duration.hours = duration.hours + int,
                        "days" => duration.days = duration.days + int,
                        "weeks" => duration.weeks = duration.weeks + int,
                        "months" => duration.months = duration.months + int,
                        "years" => duration.years = duration.years + int,
                        s => return Err(UnknownUnitError(s.to_owned())),
                    }
                }
                (Some(int), Some(dec), None, Some(unit)) => {
                    let int = BigInt::parse_bytes(int.as_str().as_bytes(), 10)
                        .ok_or_else(|| ParseIntError(int.as_str().to_owned()))?;

                    let exp = dec.as_str().len();

                    let dec = BigInt::parse_bytes(dec.as_str().as_bytes(), 10)
                        .ok_or_else(|| ParseIntError(dec.as_str().to_owned()))?;

                    // boosted_int is value * 10^exp * unit
                    let mut boosted_int = int * pow(BigInt::from(10), exp) + dec;

                    // boosted_int is now value * 10^exp * nanoseconds
                    match parse_unit(unit.as_str()) {
                        "nanoseconds" => boosted_int = boosted_int,
                        "microseconds" => boosted_int = 1_000_u64 * boosted_int,
                        "milliseconds" => boosted_int = 1_000_000_u64 * boosted_int,
                        "seconds" => boosted_int = 1_000_000_000_u64 * boosted_int,
                        "minutes" => boosted_int = 60_000_000_000_u64 * boosted_int,
                        "hours" => boosted_int = 3_600_000_000_000_u64 * boosted_int,
                        "days" => boosted_int = 86_400_000_000_000_u64 * boosted_int,
                        "weeks" => boosted_int = 604_800_000_000_000_u64 * boosted_int,
                        "months" => boosted_int = 2_629_746_000_000_000_u64 * boosted_int,
                        "years" => boosted_int = 31_556_952_000_000_000_u64 * boosted_int,
                        s => return Err(UnknownUnitError(s.to_owned())),
                    }

                    // boosted_int is now value * nanoseconds (rounding down)
                    boosted_int = boosted_int / pow(BigInt::from(10), exp);
                    duration.nanoseconds = duration.nanoseconds + boosted_int;
                }
                (Some(int), None, Some(exp), Some(unit)) => {
                    let int = BigInt::parse_bytes(int.as_str().as_bytes(), 10)
                        .ok_or_else(|| ParseIntError(int.as_str().to_owned()))?;

                    let exp = exp.as_str().parse::<isize>().or_else(|_| {
                        Err(ParseIntError(exp.as_str().to_owned()))
                    })?;

                    // boosted_int is value * 10^-exp * unit
                    let mut boosted_int = int;

                    // boosted_int is now value * 10^-exp * nanoseconds
                    match parse_unit(unit.as_str()) {
                        "nanoseconds" => boosted_int = boosted_int,
                        "microseconds" => boosted_int = 1_000_u64 * boosted_int,
                        "milliseconds" => boosted_int = 1_000_000_u64 * boosted_int,
                        "seconds" => boosted_int = 1_000_000_000_u64 * boosted_int,
                        "minutes" => boosted_int = 60_000_000_000_u64 * boosted_int,
                        "hours" => boosted_int = 3_600_000_000_000_u64 * boosted_int,
                        "days" => boosted_int = 86_400_000_000_000_u64 * boosted_int,
                        "weeks" => boosted_int = 604_800_000_000_000_u64 * boosted_int,
                        "months" => boosted_int = 2_629_746_000_000_000_u64 * boosted_int,
                        "years" => boosted_int = 31_556_952_000_000_000_u64 * boosted_int,
                        s => return Err(UnknownUnitError(s.to_owned())),
                    }

                    // boosted_int is now value * nanoseconds
                    if exp < 0 {
                        boosted_int = boosted_int / pow(BigInt::from(10), exp.abs() as usize);
                    } else {
                        boosted_int = boosted_int * pow(BigInt::from(10), exp.abs() as usize);
                    }
                    duration.nanoseconds = duration.nanoseconds + boosted_int;
                }
                (Some(int), Some(dec), Some(exp), Some(unit)) => {
                    let int = BigInt::parse_bytes(int.as_str().as_bytes(), 10)
                        .ok_or_else(|| ParseIntError(int.as_str().to_owned()))?;

                    let dec_exp = dec.as_str().len();

                    let exp = exp.as_str().parse::<isize>().or_else(|_| {
                        Err(ParseIntError(exp.as_str().to_owned()))
                    })? - (dec_exp as isize);

                    let dec = BigInt::parse_bytes(dec.as_str().as_bytes(), 10)
                        .ok_or_else(|| ParseIntError(dec.as_str().to_owned()))?;

                    // boosted_int is value * 10^-exp * unit
                    let mut boosted_int = int * pow(BigInt::from(10), dec_exp) + dec;

                    // boosted_int is now value * 10^-exp * nanoseconds
                    match parse_unit(unit.as_str()) {
                        "nanoseconds" => boosted_int = boosted_int,
                        "microseconds" => boosted_int = 1_000_u64 * boosted_int,
                        "milliseconds" => boosted_int = 1_000_000_u64 * boosted_int,
                        "seconds" => boosted_int = 1_000_000_000_u64 * boosted_int,
                        "minutes" => boosted_int = 60_000_000_000_u64 * boosted_int,
                        "hours" => boosted_int = 3_600_000_000_000_u64 * boosted_int,
                        "days" => boosted_int = 86_400_000_000_000_u64 * boosted_int,
                        "weeks" => boosted_int = 604_800_000_000_000_u64 * boosted_int,
                        "months" => boosted_int = 2_629_746_000_000_000_u64 * boosted_int,
                        "years" => boosted_int = 31_556_952_000_000_000_u64 * boosted_int,
                        s => return Err(UnknownUnitError(s.to_owned())),
                    }

                    // boosted_int is now value * nanoseconds (potentially rounded down)
                    if exp < 0 {
                        boosted_int = boosted_int / pow(BigInt::from(10), exp.abs() as usize);
                    } else {
                        boosted_int = boosted_int * pow(BigInt::from(10), exp.abs() as usize);
                    }
                    duration.nanoseconds = duration.nanoseconds + boosted_int;
                }
            }
        }
        return Ok(duration.into_duration()?);
    } else {
        // Just a unit or nothing at all
        Err(NoValueFoundError(input.to_owned()))
    }
}
