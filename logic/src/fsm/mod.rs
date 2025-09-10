use std::{
    collections::HashMap,
    fs::File,
    io::Write,
    ops::Range,
    time::{Duration, SystemTime},
};

mod draw;

use rand::Rng;

pub type BoxedStateCheck<InputValue, StateTransition, InternalState> =
    Box<dyn Fn(&Vec<InputValue>, &mut InternalState) -> Option<(usize, StateTransition)>>;

pub struct StateId {
    tag: usize,
    name: String,
    debug_name: String,
}

impl StateId {
    pub fn new(tag: usize, name: String, debug_name: String) -> Self {
        StateId {
            tag,
            name,
            debug_name,
        }
    }
}

struct StateCheck<InputKind, InputValue, StateTransition, InternalState> {
    inputs: Vec<InputKind>,
    // List of possible output states
    next_states: Vec<usize>,
    // Function to take a list of results and return the tag of the state to move to
    // or None if not changing state
    check: BoxedStateCheck<InputValue, StateTransition, InternalState>,
}

struct State<InputKind, InputValue, StateOutput, StateTransition, InternalState> {
    name: String,
    debug_name: String,
    outputs: Vec<StateOutput>,
    delay_msec: Range<u64>,
    check: StateCheck<InputKind, InputValue, StateTransition, InternalState>,
}

pub struct StateMachine<InputKind, InputValue, StateOutput, StateTransition, InternalState> {
    states:
        HashMap<usize, State<InputKind, InputValue, StateOutput, StateTransition, InternalState>>,
    current: usize,
    time: SystemTime,
    delay: Option<(Duration, usize)>,
    internal: InternalState,
    empty_input: Vec<InputKind>,
    empty_output: Vec<StateOutput>,
    graph: Option<draw::Graph>,
}

impl<InputKind, InputValue, StateOutput, StateTransition, InternalState>
    StateMachine<InputKind, InputValue, StateOutput, StateTransition, InternalState>
where
    InternalState: Default,
{
    pub fn new() -> Self {
        Self {
            states: HashMap::new(),
            current: 0,
            time: SystemTime::now(),
            delay: None,
            internal: InternalState::default(),
            empty_input: Vec::new(),
            empty_output: Vec::new(),
            graph: None,
        }
    }

    pub fn current_name(&self) -> String {
        if let Some(delay) = self.delay {
            format!("Wait<{},{:?}>", self.current_state().name, delay.0)
        } else {
            self.current_state().name.clone()
        }
    }

    pub fn debug_name(&self) -> String {
        if let Some(delay) = self.delay {
            let next = self
                .states
                .get(&delay.1)
                .expect("Couldn't get state")
                .debug_name
                .clone();
            format!(
                "Wait<{}->{},{:?}>",
                self.current_state().debug_name,
                next,
                delay.0
            )
        } else {
            self.current_state().debug_name.clone()
        }
    }

    pub fn debug_name_at_indx(&self, indx: usize) -> String {
        self.states
            .get(&indx)
            .expect("Incorrect index")
            .debug_name
            .clone()
    }

    pub fn add_state(
        &mut self,
        id: StateId,
        outputs: Vec<StateOutput>,
        delay_msec: Range<u64>,
        inputs: Vec<InputKind>,
        next_states: Vec<usize>,
        check: BoxedStateCheck<InputValue, StateTransition, InternalState>,
    ) {
        let StateId {
            tag,
            name,
            debug_name,
        } = id;
        let state = State {
            name,
            debug_name,
            outputs,
            delay_msec,
            check: StateCheck {
                inputs,
                next_states,
                check,
            },
        };
        self.states.insert(tag, state);
    }

    fn current_state(
        &self,
    ) -> &State<InputKind, InputValue, StateOutput, StateTransition, InternalState> {
        self.states
            .get(&self.current)
            .expect("Invalid current state")
    }

    pub fn inputs(&self) -> &Vec<InputKind> {
        if self.delay.is_none() {
            &self.current_state().check.inputs
        } else {
            &self.empty_input
        }
    }

    pub fn outputs(&self) -> &Vec<StateOutput> {
        if self.delay.is_none() {
            &self.current_state().outputs
        } else {
            &self.empty_output
        }
    }

    pub fn process(&mut self, inputs: Vec<InputValue>) -> Option<StateTransition> {
        if let Some(delay) = self.delay {
            let extra_delay = Duration::from_secs(0); // TODO for debug
            if self.time.elapsed().expect("Couldn't get duration") > (delay.0 + extra_delay) {
                self.current = delay.1;
                self.delay = None;
            }

            None
        } else {
            // Cannot use self.current_state() here as that would capture self immutably
            // and not allow the mutable reference to self.internal to be created
            let check = &self
                .states
                .get(&self.current)
                .expect("Invalid current state")
                .check;
            let chk_fn = &check.check;
            if let Some(next_state) = chk_fn(&inputs, &mut self.internal) {
                debug_assert!(
                    self.states.contains_key(&next_state.0),
                    "Next state ({:?}) not valid",
                    next_state.0
                );
                debug_assert!(
                    check.next_states.contains(&next_state.0),
                    "Next state ({:?}) not in list of output states",
                    next_state.0
                );
                self.time = SystemTime::now();
                let changing_state = self.current != next_state.0;
                if self.current_state().delay_msec.end == 0 {
                    if changing_state {
                        log::debug!(
                            "State: {} -> {}",
                            self.debug_name_at_indx(self.current),
                            self.debug_name_at_indx(next_state.0)
                        );
                    }
                    // No delay
                    self.current = next_state.0;
                } else if self.current_state().delay_msec.is_empty() {
                    // Delay (Fixed)
                    let duration = Duration::from_millis(self.current_state().delay_msec.start);
                    if changing_state {
                        log::debug!(
                            "State: {} -> {:?} -> {}",
                            self.debug_name_at_indx(self.current),
                            duration,
                            self.debug_name_at_indx(next_state.0)
                        );
                    }
                    self.delay = Some((duration, next_state.0))
                } else {
                    // Delay (Random)
                    let mut rng = rand::rng();
                    let delay = rng.random_range(self.current_state().delay_msec.clone());
                    let duration = Duration::from_millis(delay);
                    if changing_state {
                        log::debug!(
                            "State: {} -> {:?} -> {}",
                            self.debug_name_at_indx(self.current),
                            duration,
                            self.debug_name_at_indx(next_state.0)
                        );
                    }
                    self.delay = Some((duration, next_state.0))
                }

                Some(next_state.1)
            } else {
                None
            }
        }
    }

    pub fn graph_str(&self) -> String {
        let graph_config = "ranksep=0.1; bgcolor=\"#7A151B\"; dpi=72; size=\"4.6,7!\";";
        let node_config = "shape=rect,height=0.1,style=filled,color=\"#D48735\",fillcolor=\"#FBCF9D\",fontcolor=\"#7A151B\",fontsize=8,fontname=\"DejaVu Sans Mono\"";
        let edge_config = "arrowsize=0.5,color=\"#D48735\"";
        let mut graph = format!("digraph {{\n  {}\n", graph_config);

        let mut state_list: Vec<&usize> = self.states.keys().collect();
        state_list.sort();

        for state in state_list {
            graph.push_str(&format!(
                "  {} [label=\"{}\",id=\"shaoooh_{}\",{}];\n",
                state,
                self.states.get(state).unwrap().name,
                state,
                node_config
            ));
        }

        for state in &self.states {
            for next in &state.1.check.next_states {
                graph.push_str(&format!("  {} -> {} [{}];\n", state.0, next, edge_config));
            }
        }

        graph.push('}');
        graph
    }

    pub fn graph_file(&mut self, file_root: &str) -> Result<(), Box<dyn std::error::Error>> {
        // TODO cleanly error if dot/inkscape not installed
        let dot_file_name = format!("{}.dot", file_root);
        let svg_file_name = format!("{}.svg", file_root);
        let png_file_name = format!("{}.png", file_root);
        let mut sorted_keys: Vec<&usize> = self.states.keys().collect();
        sorted_keys.sort();
        let state_ids: Vec<String> = sorted_keys
            .into_iter()
            .map(|f| format!("shaoooh_{}", f))
            .collect();

        let mut file = File::create(&dot_file_name)?;
        file.write_all(self.graph_str().as_bytes())?;
        file.flush()?;

        let svg = std::process::Command::new("dot")
            .arg("-Tsvg")
            .arg(&dot_file_name)
            .output()?;

        let mut file = File::create(&svg_file_name).unwrap();
        file.write_all(&svg.stdout).unwrap();
        file.flush().unwrap();

        std::process::Command::new("inkscape")
            .arg(&svg_file_name)
            .arg("-o")
            .arg(&png_file_name)
            .status()?;

        let svg_info = std::process::Command::new("inkscape")
            .arg(svg_file_name)
            .arg("-I")
            .arg(state_ids.join(","))
            .arg("-X")
            .arg("-Y")
            .arg("-W")
            .arg("-H")
            .output()?;

        let stdout = String::from_utf8(svg_info.stdout)?;
        let svg_info_arrays: Vec<&str> = stdout.split('\n').collect();
        let x_array = svg_info_arrays[0]
            .split(',')
            .map(|f| f.parse::<f32>().unwrap().round() as i32)
            .collect();
        let y_array = svg_info_arrays[1]
            .split(',')
            .map(|f| f.parse::<f32>().unwrap().round() as i32)
            .collect();
        let w_array = svg_info_arrays[2]
            .split(',')
            .map(|f| f.parse::<f32>().unwrap().round() as i32)
            .collect();
        let h_array = svg_info_arrays[3]
            .split(',')
            .map(|f| f.parse::<f32>().unwrap().round() as i32)
            .collect();

        self.graph = Some(draw::Graph::new(
            png_file_name.as_str(),
            x_array,
            y_array,
            w_array,
            h_array,
        ));

        Ok(())
    }

    pub fn graph_with_state(&self) -> Option<opencv::core::Mat> {
        self.graph.as_ref().map(|g| g.with_state(self.current))
    }
}

impl<InputKind, InputValue, StateOutput, StateTransition, InternalState> Default
    for StateMachine<InputKind, InputValue, StateOutput, StateTransition, InternalState>
where
    InternalState: Default,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<InputKind, InputValue, StateOutput, StateTransition, InternalState> std::fmt::Debug
    for StateMachine<InputKind, InputValue, StateOutput, StateTransition, InternalState>
where
    InputKind: std::fmt::Debug,
    InputValue: std::fmt::Debug,
    StateOutput: std::fmt::Debug,
    StateTransition: std::fmt::Debug,
    InternalState: std::fmt::Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("StateMachine")
            .field("states", &self.states)
            .field("current", &self.current)
            .field("internal", &self.internal)
            .finish()
    }
}

impl<InputKind, InputValue, StateOutput, StateTransition, InternalState> std::fmt::Debug
    for State<InputKind, InputValue, StateOutput, StateTransition, InternalState>
where
    InputKind: std::fmt::Debug,
    InputValue: std::fmt::Debug,
    StateOutput: std::fmt::Debug,
    StateTransition: std::fmt::Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("State")
            .field("name", &self.name)
            .field("debug_name", &self.debug_name)
            .field("outputs", &self.outputs)
            .field("delay_msec", &self.delay_msec)
            .field("check", &self.check)
            .finish()
    }
}

impl<InputKind, InputValue, StateTransition, InternalState> std::fmt::Debug
    for StateCheck<InputKind, InputValue, StateTransition, InternalState>
where
    InputKind: std::fmt::Debug,
    InputValue: std::fmt::Debug,
    StateTransition: std::fmt::Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("StateCheck")
            .field("inputs", &self.inputs)
            .field("next_states", &self.next_states)
            .finish()
    }
}
