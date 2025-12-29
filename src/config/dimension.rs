use crate::prelude::*;
use logos::Logos;
use ratatui::layout::Constraint;
use std::str::FromStr;

#[derive(Debug, Clone)]
pub enum Dimension {
    Length(u16),
    Percentage(u16),
}

#[derive(logos::Logos)]
#[logos(skip r"[ \t\n\f]+")]
enum DimensionToken {
    #[token("%")]
    UnitPercent,

    #[regex("length")]
    UnitLength,

    #[regex("[0-9]+")]
    Number,
}

impl FromStr for Dimension {
    type Err = ConfigError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut lexer = DimensionToken::lexer(s);
        use DimensionToken as T;

        let Some(Ok(T::Number)) = lexer.next() else {
            return Err(ConfigError::DimensionParseError(
                "expected number".to_owned(),
            ));
        };

        let number = lexer.slice().parse::<u16>().map_err(|_| {
            ConfigError::DimensionParseError("unable to parse number (too large?)".to_owned())
        })?;

        let dimension = match lexer.next() {
            Some(Ok(T::UnitLength)) => Ok(Dimension::Length(number)),
            Some(Ok(T::UnitPercent)) => {
                if number.clamp(0, 100) != number {
                    Err(ConfigError::DimensionParseError(
                        "percent value must be between 0 and 100".to_owned(),
                    ))
                } else {
                    Ok(Dimension::Percentage(number))
                }
            }
            _ => Err(ConfigError::DimensionParseError(
                "expecting unit".to_owned(),
            )),
        }?;

        if lexer.next().is_some() {
            Err(ConfigError::DimensionParseError(
                "invalid trailing characters after dimension".to_owned(),
            ))
        } else {
            Ok(dimension)
        }
    }
}

impl Dimension {
    pub fn as_constraint(&self) -> Constraint {
        use Dimension as D;
        match self {
            D::Length(length) => Constraint::Length(*length),
            D::Percentage(percent) => Constraint::Percentage(*percent),
        }
    }

    pub fn as_complementary_constraint(&self, max: u16) -> Constraint {
        use Dimension as D;
        match self {
            D::Length(length) => Constraint::Length(max.saturating_sub(*length)),
            D::Percentage(percent) => Constraint::Percentage(100u16.saturating_sub(*percent)),
        }
    }
}

impl<'de> serde::Deserialize<'de> for Dimension {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Dimension::from_str(&s).map_err(|err| serde::de::Error::custom(err.to_string()))
    }
}

#[cfg(test)]
mod tests {

    macro_rules! dimension_tests {
        ($($name:ident: $str:literal => $val:pat,)*) => {
            $(
                #[test]
                fn $name() {
                    let mut deserializer =
                        serde_assert::Deserializer::builder([Token::Str($str.to_owned())]).build();
                    claims::assert_matches!(
                        Dimension::deserialize(&mut deserializer),
                        $val
                    );
                }

            )*
        }
    }

    use serde::Deserialize;
    use serde_assert::Token;

    use super::*;

    dimension_tests! {

        no_space_length: "10length" => Ok(Dimension::Length(10)),
        space_length: "4 length" => Ok(Dimension::Length(4)),
        multiple_spaces_length: "  19 \t length  \t" => Ok(Dimension::Length(19)),
        leading_zeros_length: "  00019 \t length  \t" => Ok(Dimension::Length(19)),
        zero_length: "0 length" => Ok(Dimension::Length(0)),

        no_space_percent: "81%" => Ok(Dimension::Percentage(81)),
        space_percent: "10 %" => Ok(Dimension::Percentage(10)),
        multiple_spaces_percent: "  9 \t %  \t" => Ok(Dimension::Percentage(9)),
        leading_zeros_percent: "  00019 \t %  \t" => Ok(Dimension::Percentage(19)),
        zero_percent: "0 %" => Ok(Dimension::Percentage(0)),
        hundred_percent: "100 %" => Ok(Dimension::Percentage(100)),

        garbage: "abc" => Err(_),
        empty: "" => Err(_),
        just_length: "length" => Err(_),
        just_percent: "percent" => Err(_),
        negative_length: "-1length" => Err(_),
        negative_percent: "-1%" => Err(_),
        too_much_percent: "101%" => Err(_),
    }
}
