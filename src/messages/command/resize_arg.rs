use crate::prelude::*;
use std::str::FromStr;

#[derive(Debug, Copy, Clone, Default)]
pub enum ResizeArg {
    Increase(Dimension),
    Decrease(Dimension),
    Absolute(Dimension),
    #[default]
    Default,
}

impl ResizeArg {
    pub fn apply(self, dimension: Dimension, default: Dimension, max: u16) -> Dimension {
        use ResizeArg as A;

        match self {
            A::Increase(increase) => dimension.add(increase, max),
            A::Decrease(decrease) => dimension.subtract(decrease, max),
            A::Absolute(absolute) => absolute,
            A::Default => default,
        }
    }
}

impl FromStr for ResizeArg {
    type Err = CommandParseError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        use ResizeArg as A;

        let s = s.trim();

        Ok(match s.chars().nth(0) {
            Some('+') => A::Increase(
                Dimension::from_str(&s[1..]).map_err(|_| CommandParseError::ResizeArgExpected)?,
            ),
            Some('-') => A::Decrease(
                Dimension::from_str(&s[1..]).map_err(|_| CommandParseError::ResizeArgExpected)?,
            ),
            Some(_) => A::Absolute(
                Dimension::from_str(s).map_err(|_| CommandParseError::ResizeArgExpected)?,
            ),
            None => A::Default,
        })
    }
}

#[cfg(test)]
mod tests {

    macro_rules! dimension_tests {
        ($($name:ident: $str:literal => $val:pat,)*) => {
            $(
                #[test]
                fn $name() {
                    claims::assert_matches!(
                        ResizeArg::from_str($str),
                        $val
                    );
                }

            )*
        }
    }

    use super::*;

    use Dimension as D;
    use ResizeArg as A;

    dimension_tests! {

        absolute_length: "10 length" => Ok(A::Absolute(D::Length(10)) ),
        absolute_percent: "10%" => Ok(A::Absolute(D::Percentage(10)) ),
        absolute_with_spaces: "  10%" => Ok(A::Absolute(D::Percentage(10)) ),

        relative_inc_length: "+10 length" => Ok(A::Increase(D::Length(10)) ),
        relative_inc_percent: "+10%" => Ok(A::Increase(D::Percentage(10)) ),
        relative_inc_with_spaces: "  +10%" => Ok(A::Increase(D::Percentage(10)) ),
        space_between_plus_and_dimension: "+ 10%" => Ok(A::Increase(D::Percentage(10)) ),

        relative_dec_length: "-10 length" => Ok(A::Decrease(D::Length(10)) ),
        relative_dec_percent: "-10%" => Ok(A::Decrease(D::Percentage(10)) ),
        relative_dec_with_spaces: "  -10%" => Ok(A::Decrease(D::Percentage(10)) ),

        default: "" => Ok(A::Default),
        default_with_spaces: "  \t \n   \r" => Ok(A::Default),

        garbage: "abc" => Err(CommandParseError::ResizeArgExpected),
        no_dimension_plus: "+" => Err(CommandParseError::ResizeArgExpected),
        no_dimension_minus: "-" => Err(CommandParseError::ResizeArgExpected),
        two_pluses: "++" => Err(CommandParseError::ResizeArgExpected),
        two_minuses_with_space: "- -" => Err(CommandParseError::ResizeArgExpected),
        plus_minus: "+-" => Err(CommandParseError::ResizeArgExpected),
        trailing_content: "+10% units" => Err(CommandParseError::ResizeArgExpected),
        minus_plus_percent: "-+10%" => Err(CommandParseError::ResizeArgExpected),
    }
}
