use std::ops::Range;

#[derive(Debug, PartialEq, Clone)]
pub enum InternalOp {
    SetCounter(usize),
    StartTimer,
    UpdateTimer,
    LastDelay(u64),
    ProcDelay(u64),
    ProcDelayRange(u64, Range<u64>),
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
    IncrEncounters,
}

impl InternalOp {
    fn is_branch(&self) -> bool {
        matches!(
            self,
            InternalOp::LastDelay(..)
                | InternalOp::CounterZero
                | InternalOp::CounterValue(..)
                | InternalOp::CheckToggle
                | InternalOp::Deadend
        )
    }

    fn is_proc_mod(&self) -> bool {
        matches!(
            self,
            InternalOp::ProcDelay(..) | InternalOp::ProcDelayRange(..)
        )
    }
}

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum InputOp {
    And,
    Or,
}

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum GroupOp {
    And,
    AndNot,
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
    // TODO support for different input ops
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
            grp1_2_op: GroupOp::And,
        }
    }

    pub fn add_tag(&mut self, s: String) {
        self.tag = Some(s);
    }

    pub fn tag(&self) -> Option<&str> {
        self.tag.as_deref()
    }

    pub fn set_delay(&mut self, d: Range<u64>) {
        self.delay = d;
    }

    pub fn delay(&self) -> Range<u64> {
        self.delay.clone()
    }

    pub fn set_outputs(&mut self, o: Vec<String>) {
        self.outputs = o;
    }

    pub fn outputs(&self) -> Vec<&str> {
        self.outputs.iter().map(|s| s as &str).collect()
    }

    pub fn set_modifiers(&mut self, m: Vec<InternalOp>) {
        if !m.is_empty() {
            self.simple = false;
            self.internal_ops = m;
        }
    }

    pub fn modifiers(&self) -> &[InternalOp] {
        &self.internal_ops
    }

    pub fn is_deadend(&self) -> bool {
        for i in &self.internal_ops {
            if matches!(i, InternalOp::Deadend) {
                return true;
            }
        }
        false
    }

    pub fn set_positive(&mut self, b: String) {
        self.simple = false;
        self.tag_met = Some(b);
    }

    pub fn positive(&self) -> Option<&str> {
        self.tag_met.as_deref()
    }

    pub fn negative(&self) -> Option<&str> {
        self.tag_not_met.as_deref()
    }

    pub fn set_negative(&mut self, b: String) {
        self.simple = false;
        self.tag_not_met = Some(b);
    }

    pub fn set_processing(&mut self, p: (&str, Option<(GroupOp, &str)>)) {
        self.simple = false;
        self.processing_grp1 = p.0.split(',').map(|s| (*s).into()).collect();
        if let Some((grp_op, grp2)) = p.1 {
            self.processing_grp2 = grp2.split(',').map(|s| (*s).into()).collect();
            self.grp1_2_op = grp_op;
        }
    }

    pub fn any_branch(&self) -> bool {
        self.internal_ops.iter().any(|x| x.is_branch())
    }

    pub fn single_branch(&self) -> bool {
        self.internal_ops.iter().filter(|x| x.is_branch()).count() <= 1
    }

    pub fn get_branch(&self) -> Option<InternalOp> {
        self.internal_ops.iter().find(|x| x.is_branch()).cloned()
    }

    pub fn any_proc_mod(&self) -> bool {
        self.internal_ops.iter().any(|x| x.is_proc_mod())
    }

    pub fn single_proc_mod(&self) -> bool {
        self.internal_ops.iter().filter(|x| x.is_proc_mod()).count() <= 1
    }

    pub fn get_proc_mod(&self) -> Option<InternalOp> {
        self.internal_ops.iter().find(|x| x.is_proc_mod()).cloned()
    }

    pub fn any_processing(&self) -> bool {
        !self.processing_grp1.is_empty() || !self.processing_grp2.is_empty()
    }

    pub fn inputs_grp1(&self) -> Vec<&str> {
        self.processing_grp1.iter().map(|s| s as &str).collect()
    }

    pub fn inputs_grp2(&self) -> Vec<&str> {
        self.processing_grp2.iter().map(|s| s as &str).collect()
    }

    pub fn grp1_op(&self) -> InputOp {
        self.grp1_op
    }

    pub fn grp2_op(&self) -> InputOp {
        self.grp2_op
    }

    pub fn grp1_2_op(&self) -> GroupOp {
        self.grp1_2_op
    }

    pub fn simple(&self) -> bool {
        self.simple
    }

    fn err_name(&self) -> &str {
        match &self.tag {
            Some(s) => s,
            None => "#anonymous#",
        }
    }

    pub fn is_ok(&self) -> bool {
        let branch_and_proc = self.any_processing() && self.any_branch();
        let single_branch = self.single_branch();
        let single_proc_mod = self.single_proc_mod();
        let proc_mod_and_no_proc = self.any_proc_mod() && !self.any_processing();

        if branch_and_proc {
            log::error!(
                "State '{}' contains both a branch and processing steps",
                self.err_name()
            );
        }
        if !single_branch {
            log::error!("State '{}' contains multiple branches", self.err_name());
        }
        if !single_proc_mod {
            log::error!(
                "State '{}' contains multiple processing modifiers",
                self.err_name()
            );
        }
        if proc_mod_and_no_proc {
            log::error!(
                "State '{}' contains processing modifier but no processing",
                self.err_name()
            );
        }

        !branch_and_proc && single_branch && single_proc_mod && !proc_mod_and_no_proc
    }
}
