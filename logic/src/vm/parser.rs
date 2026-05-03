use nom::branch::alt;
use nom::bytes::complete::take_while;
use nom::bytes::tag;
use nom::bytes::{is_not, take_until1};
use nom::character::complete::space0;
use nom::character::satisfy;
use nom::combinator::{all_consuming, complete, eof, map, map_res, opt, recognize};
use nom::multi::{many0, many1_count};
use nom::sequence::{delimited, preceded};
use nom::IResult;
use nom::{AsChar, Parser};

use handlebars::Handlebars;

use std::ops::Range;

use crate::vm::state::InternalOp;
use crate::vm::state::{State, GroupOp};
use crate::vm::state_machine::ParsedStateMachine;

use serde::Serialize;

use crate::app::{Game, Method};

pub struct FsmParser {
    fsm: Vec<u8>,
}

const LABEL: &str = "@";
const COMMENT: &str = "#";
const STATE: &str = "State";
const MODIFIER: &str = "&";

enum ParseType {
    PositiveBranch,
    NegativeBranch,
    Delay,
    Processing,
}

#[derive(Serialize)]
struct TplContext {
    target: u32,
    game: Game,
    method: Method,
}

impl FsmParser {
    pub fn new(mut fsm: Vec<u8>) -> Self {
        fsm.push(b'\n'); // Ensure empty newline at end
        Self { fsm }
    }

    fn end_or_newline(s: &str) -> IResult<&str, &str> {
        alt((tag("\n"), eof)).parse(s)
    }

    fn parse_comment(s: &str) -> IResult<&str, Option<State>> {
        let (_input, (_tag, _, comment)) = (tag(COMMENT), space0, is_not("\n")).parse(s)?;

        Ok((comment, None))
    }

    fn parse_tag(s: &str) -> IResult<&str, Option<&str>> {
        opt(complete(map((take_until1(LABEL), tag(LABEL)), |(a, _b)| a))).parse(s)
    }

    fn alphanumeric_or_equals(s: &str) -> IResult<&str, char> {
        satisfy(|x| AsChar::is_alphanum(x) || x == '=').parse(s)
    }

    fn usize_from_text(s: &str) -> IResult<&str, usize> {
        map_res(take_while(AsChar::is_dec_digit), |s| {
            usize::from_str_radix(s, 10)
        })
        .parse(s)
    }

    fn u64_from_text(s: &str) -> IResult<&str, u64> {
        map_res(take_while(AsChar::is_dec_digit), |s| {
            u64::from_str_radix(s, 10)
        })
        .parse(s)
    }

    fn parse_state_modifier_enum(s: &str) -> IResult<&str, InternalOp> {
        alt((
            map((tag("SetCounter="), Self::usize_from_text), |(_, n)| {
                InternalOp::SetCounter(n)
            }),
            map(tag("StartTimer"), |_| InternalOp::StartTimer),
            map(tag("UpdateTimer"), |_| InternalOp::UpdateTimer),
            map((tag("LastDelay="), Self::u64_from_text), |(_, n)| {
                InternalOp::LastDelay(n)
            }),
            map((tag("ProcDelay="), Self::u64_from_text), |(_, n)| {
                InternalOp::ProcDelay(n)
            }),
            map(
                (
                    tag("ProcDelayRange="),
                    Self::u64_from_text,
                    tag(","),
                    Self::u64_from_text,
                    tag(".."),
                    Self::u64_from_text,
                ),
                |(_, n, _, a, _, b)| InternalOp::ProcDelayRange(n, a..b),
            ),
            map(tag("CounterZero"), |_| InternalOp::CounterZero),
            map((tag("CounterValue="), Self::usize_from_text), |(_, n)| {
                InternalOp::CounterValue(n)
            }),
            map(tag("DecrCounter"), |_| InternalOp::DecrCounter),
            map(tag("IncrCounter"), |_| InternalOp::IncrCounter),
            map(tag("CheckToggle"), |_| InternalOp::CheckToggle),
            map(tag("Toggle"), |_| InternalOp::Toggle),
            map(tag("SetAtomic"), |_| InternalOp::SetAtomic),
            map(tag("ClearAtomic"), |_| InternalOp::ClearAtomic),
            map(tag("Deadend"), |_| InternalOp::Deadend),
            map(tag("FoundTarget"), |_| InternalOp::FoundTarget),
            map(tag("IncrEncounters"), |_| InternalOp::IncrEncounters),
        ))
        .parse(s)
    }

    fn parse_state_modifier(s: &str) -> IResult<&str, InternalOp> {
        let (input, (_, modifier)) = (
            tag(MODIFIER),
            recognize(many1_count(Self::alphanumeric_or_equals)),
        )
            .parse(s)?;

        let (_, modifier_enum) = Self::parse_state_modifier_enum(modifier)?;

        Ok((input, modifier_enum))
    }

    fn parse_state_modifiers(s: &str) -> IResult<&str, Vec<InternalOp>> {
        many0(Self::parse_state_modifier).parse(s)
    }

    fn middle_parse_type(c: Option<char>) -> Option<ParseType> {
        if let Some(c) = c {
            match c {
                '+' => Some(ParseType::PositiveBranch),
                '-' => Some(ParseType::NegativeBranch),
                s if ('0'..'9').contains(&s) => Some(ParseType::Delay),
                '{' => Some(ParseType::Processing),
                _ => None,
            }
        } else {
            None
        }
    }

    fn parse_positive_branch(s: &str) -> IResult<&str, &str> {
        let (remaining, (_, b)) = complete((tag("+"), alt((take_until1(" "), take_until1("\n"))))).parse(s)?;

        Ok((remaining, b))
    }

    fn parse_negative_branch(s: &str) -> IResult<&str, &str> {
        let (remaining, (_, b)) = complete((tag("-"), alt((take_until1(" "), take_until1("\n"))))).parse(s)?;

        Ok((remaining, b))
    }

    fn parse_delay(s: &str) -> IResult<&str, Range<u64>> {
        alt((
            map(
                (Self::u64_from_text, tag(".."), Self::u64_from_text),
                |(a, _, b)| a..b,
            ),
            map(Self::u64_from_text, |n| n..n),
        ))
        .parse(s)
    }

    fn parse_processing(s: &str) -> IResult<&str, (&str, Option<(GroupOp, &str)>)> {
        (
            delimited(tag("{"), take_until1("}"), tag("}")),
            opt((
                alt((map(tag("&!"), |_| GroupOp::AndNot),
                    map(tag("&"), |_| GroupOp::And))),
                delimited(tag("{"), take_until1("}"), tag("}")),
            )),
        )
            .parse(s)
    }

    fn state_middle(
        s: &str,
    ) -> IResult<
        &str,
        (
            Range<u64>,
            Option<&str>,
            Option<&str>,
            Option<(&str, Option<(GroupOp, &str)>)>,
        ),
    > {
        let mut delay = 0..0;
        let mut positive_branch = None;
        let mut negative_branch = None;
        let mut processing = None;

        let mut remaining = s;

        log::trace!("Parsing middle: '{}'", s);

        while let Some(t) = Self::middle_parse_type(remaining.chars().next()) {
            match t {
                ParseType::PositiveBranch => {
                    let p = Self::parse_positive_branch(remaining)?;
                    remaining = p.0.trim_start_matches(' ');
                    positive_branch = Some(p.1);
                }
                ParseType::NegativeBranch => {
                    let n = Self::parse_negative_branch(remaining)?;
                    remaining = n.0.trim_start_matches(' ');
                    negative_branch = Some(n.1);
                }
                ParseType::Delay => {
                    let d = Self::parse_delay(remaining)?;
                    remaining = d.0.trim_start_matches(' ');
                    delay = d.1;
                }
                ParseType::Processing => {
                    let p = Self::parse_processing(remaining)?;
                    remaining = p.0.trim_start_matches(' ');
                    processing = Some(p.1);
                }
            }
        }

        log::trace!("Remains: '{}'", remaining);

        Ok((
            remaining,
            (delay, positive_branch, negative_branch, processing),
        ))
    }

    fn valid_output_char(s: &str) -> IResult<&str, char> {
        satisfy(|x| x != '\n' && x != ':').parse(s)
    }

    fn parse_outputs(s: &str) -> IResult<&str, Vec<&str>> {
        log::trace!("Parsing '{}' for outputs", s);
        many0(preceded(
            tag(":"),
            recognize(many1_count(Self::valid_output_char)),
        ))
        .parse(s)
    }

    fn parse_state(s: &str) -> IResult<&str, State> {
        // Optional label
        // State (with optional state modifiers)
        // In any order:
        //  - Delay (int or range)
        //  - Processing
        //  - Positive branch
        //  - Negative branch
        // Optional output(s)
        let (input, (tag, _, _, modifiers, _, middle, _, outputs, _)) = all_consuming(complete((
            Self::parse_tag,
            space0,
            tag(STATE),
            Self::parse_state_modifiers,
            space0,
            Self::state_middle,
            space0,
            Self::parse_outputs,
            Self::end_or_newline,
        )))
        .parse(s)?;

        // log::warn!(
        //     "Think got state: {:?}\nmods = {:?}, mid = {:?}\noutputs = {:?}",
        //     tag,
        //     modifiers,
        //     middle,
        //     outputs
        // );

        let mut state = State::new();

        if let Some(tag) = tag {
            state.add_tag(tag.to_string());
        }

        let (delay, positive, negative, processing) = middle;

        state.set_delay(delay);

        state.set_outputs(outputs.into_iter().map(|s| s.to_string()).collect());
        state.set_modifiers(modifiers);

        if let Some(positive) = positive {
            state.set_positive(positive.to_string());
        }

        if let Some(negative) = negative {
            state.set_negative(negative.to_string());
        }

        if let Some(processing) = processing {
            state.set_processing(processing);
        }

        //          (delay, positive_branch, negative_branch, processing),

        Ok((input, state))
    }

    fn parse_line(s: &str) -> IResult<&str, Option<State>> {
        alt((
            map(Self::end_or_newline, |_| None),
            map(Self::parse_comment, |_| None),
            map(Self::parse_state, |s| Some(s)),
        ))
        .parse(s)
    }

    fn preprocess(&self) -> Option<String> {
        if let Ok(s) = std::str::from_utf8(&self.fsm) {
            let mut hbars = Handlebars::new();
            hbars.set_strict_mode(true);

            let data = TplContext {
                target: 1,
                game: Game::FireRedLeafGreen,
                method: Method::SoftResetGift
            };

            match hbars.render_template(s, &data) {
                Ok(rndr) => Some(rndr),
                Err(e) => {
                    log::error!("Failed to preprocess: {}", e);
                    None
                }
            }
        } else {
            log::error!("Failed to read as utf-8");
            None
        }
    }

    pub fn parse(self) -> Option<ParsedStateMachine> {
        let mut any_error = false;
        let mut result = Vec::new();
        let input = self.preprocess()?;
        for line in input.split_inclusive('\n') {
            let lstr = line.trim_start_matches(' ');
            log::trace!("Parsing: {}", lstr);
            let p = Self::parse_line(lstr);
            if let Ok(p) = p {
                if let Some(s) = p.1 {
                    if !s.is_ok() {
                        log::error!("State not ok: {:?}", s);
                        any_error = true;
                    }
                    result.push(s);
                }
            } else {
                log::error!("Failed to parse: '{}' ({:?})", lstr, p);
                any_error = true;
            }
        }

        if any_error {
            None
        } else {
            Some(ParsedStateMachine::new(result))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_comment() {
        let inp = "# this is a comment\n";
        let res = FsmParser::parse_comment(inp).unwrap();
        assert_eq!(res, ("this is a comment", None));
    }

    #[test]
    fn parse_valid_tag() {
        let inp = "tagname@";
        let res = FsmParser::parse_tag(inp).unwrap();
        assert_eq!(res, ("", Some("tagname")));
    }

    #[test]
    fn parse_tag_trailing() {
        let inp = "tagname@this is trailing";
        let res = FsmParser::parse_tag(inp).unwrap();
        assert_eq!(res, ("this is trailing", Some("tagname")));

        let inp = "tagname@    this is trailing";
        let res = FsmParser::parse_tag(inp).unwrap();
        assert_eq!(res, ("    this is trailing", Some("tagname")));
    }

    #[test]
    fn parse_underscore_tag() {
        let inp = "tag_na_me@";
        let res = FsmParser::parse_tag(inp).unwrap();
        assert_eq!(res, ("", Some("tag_na_me")));
    }

    #[test]
    fn parse_no_tag() {
        let inp = "this is trailing";
        let res = FsmParser::parse_tag(inp).unwrap();
        assert_eq!(res, ("this is trailing", None));

        let inp = "";
        let res = FsmParser::parse_tag(inp).unwrap();
        assert_eq!(res, ("", None));
    }

    #[test]
    fn parse_no_state_modifier() {
        let inp = " trailing";
        let res = FsmParser::parse_state_modifiers(inp).unwrap();
        assert_eq!(res, (" trailing", vec![]));
    }

    #[test]
    fn parse_1_state_modifier() {
        let inp = "&UpdateTimer trailing";
        let res = FsmParser::parse_state_modifiers(inp).unwrap();
        assert_eq!(res, (" trailing", vec![InternalOp::UpdateTimer]));
    }

    #[test]
    fn parse_2_state_modifier() {
        let inp = "&UpdateTimer&IncrEncounters trailing";
        let res = FsmParser::parse_state_modifiers(inp).unwrap();
        assert_eq!(
            res,
            (
                " trailing",
                vec![InternalOp::UpdateTimer, InternalOp::IncrEncounters]
            )
        );
    }

    #[test]
    fn parse_3_state_modifier() {
        let inp = "&UpdateTimer&SetCounter=5&IncrEncounters trailing";
        let res = FsmParser::parse_state_modifiers(inp).unwrap();
        assert_eq!(
            res,
            (
                " trailing",
                vec![
                    InternalOp::UpdateTimer,
                    InternalOp::SetCounter(5),
                    InternalOp::IncrEncounters
                ]
            )
        );
    }
}
