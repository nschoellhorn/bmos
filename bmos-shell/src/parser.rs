use alloc::vec::Vec;
use nom::IResult;

const ALLOWED_CHARS: &'static str = ".:/-_";

pub fn parse_command_line(input: &str) -> IResult<&str, Vec<&str>> {
    use nom::branch::alt;
    use nom::character::complete::multispace1;
    use nom::multi::separated_list0;

    separated_list0(multispace1, alt((allowed_character, quoted_string)))(input)
}

fn allowed_character(input: &str) -> IResult<&str, &str> {
    use nom::bytes::complete::take_while1;
    use nom::character::is_alphanumeric;

    take_while1(|c: char| is_alphanumeric(c as u8) || ALLOWED_CHARS.contains(c))(input)
}

fn quoted_string(input: &str) -> IResult<&str, &str> {
    use nom::branch::alt;

    alt((double_quoted_string, single_quoted_string))(input)
}

fn double_quoted_string(input: &str) -> IResult<&str, &str> {
    use nom::bytes::complete::escaped;
    use nom::bytes::complete::is_not;
    use nom::character::complete::char;
    use nom::sequence::delimited;

    delimited(
        char('"'),
        escaped(is_not("\\\""), '\\', char('"')),
        char('"'),
    )(input)
}

fn single_quoted_string(input: &str) -> IResult<&str, &str> {
    use nom::bytes::complete::escaped;
    use nom::bytes::complete::is_not;
    use nom::character::complete::char;
    use nom::sequence::delimited;

    delimited(
        char('\''),
        escaped(is_not("\\\'"), '\\', char('\'')),
        char('\''),
    )(input)
}
