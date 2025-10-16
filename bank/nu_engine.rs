use nu_engine::eval_block;
use nu_parser::parse;
use nu_protocol::{
    engine::{EngineState, Stack},
    Value,
};
use nu_source::Text;

fn main() {
    // Initialize the engine state and stack
    let mut engine_state = EngineState::new();
    let mut stack = Stack::new();

    // Example command string
    let command_string = "echo 'Hello, world!'";

    // Parse the command
    let (block, delta) = {
        let source = Text::from(command_string);
        parse(&mut engine_state, &source, 0)
    };

    // Apply the parsed delta to the engine state
    if let Some(delta) = delta {
        engine_state.merge_delta(delta);
    }

    // Evaluate the block
    if let Some(block) = block {
        match eval_block(&engine_state, &mut stack, &block, Value::nothing()) {
            Ok(value) => {
                println!("Result: {:?}", value);
            }
            Err(err) => {
                println!("Error: {:?}", err);
            }
        }
    }
}
