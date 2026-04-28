
use std::ops::Range;

#[derive(Debug, PartialEq)]
pub enum InternalOp {
    SetCounter(usize),
    StartTimer,
    UpdateTimer,
    LastDelay(u64),
    LastDelayRange(u64, Range<u64>),
    CounterZero,
    CounterValue(usize),
    DecrCounter,
    IncrCounter,
    CheckToggle,
    Toggle,
    SetAtomic,
    ClearAtomic,
    Deadend,
    FoundTarget,
    IncrEncounters
}

#[derive(Debug, PartialEq)]
pub enum InputOp {
    And,
    Or
}

#[derive(Debug, PartialEq)]
pub enum GroupOp {
    And,
    AndNot
}

#[derive(Debug, PartialEq)]
pub struct State {
    tag: Option<String>,
    delay: Range<u64>,
    outputs: Vec<String>,
    simple: bool,
    internal_ops: Vec<InternalOp>,
    tag_met: Option<String>,
    tag_not_met: Option<String>,
    processing_grp1: Vec<String>,
    processing_grp2: Vec<String>,
    grp1_op: InputOp,
    grp2_op: InputOp,
    grp1_2_op: GroupOp,
}

impl State {
    pub fn new() -> Self {
        Self {
            tag: None,
            delay: 0..0,
            outputs: vec![],
            simple: true,
            internal_ops: vec![],
            tag_met: None,
            tag_not_met: None,
            processing_grp1: vec![],
            processing_grp2: vec![],
            grp1_op: InputOp::And,
            grp2_op: InputOp::And,
            grp1_2_op: GroupOp::And
        }
    }

    pub fn add_tag(&mut self, s: String) {
        self.tag = Some(s);
    }

    pub fn set_delay(&mut self, d : Range<u64>) {
        self.delay = d;
    }

    pub fn set_outputs(&mut self, o : Vec<String>) {
        self.outputs = o;
    }

    pub fn set_modifiers(&mut self, m : Vec<InternalOp>) {
        self.simple = false;
        self.internal_ops = m;
    }

    pub fn set_positive(&mut self, b : String) {
        self.simple = false;
        self.tag_met = Some(b);
    }
    
    pub fn set_negative(&mut self, b : String) {
        self.simple = false;
        self.tag_not_met = Some(b);
    }

    pub fn set_processing(&mut self, p : (&str, Option<(GroupOp, &str)>)) {
        self.simple = false;
        self.processing_grp1 = p.0.split(',').map(|s| (*s).into()).collect();
        if let Some((grp_op, grp2)) = p.1 {
            self.processing_grp2 = grp2.split(',').map(|s| (*s).into()).collect();
            self.grp1_2_op = grp_op;
        }
    }

}