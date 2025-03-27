use crate::elevator::orders;
use crate::elevator::state;


use serde_json;
use std::process;

pub fn assign_orders(
    states: &state::States,
    cab_requests: &orders::CabOrders,
    hall_requests: &orders::HallOrders,
) -> String {
    
    let mut states_map = serde_json::Map::new();

    for (id, state) in states.iter().enumerate() {
        if !state.is_availible() {
            continue;
        }
        let state_variables = serde_json::json!({
            "behaviour": state.behaviour.to_string(),
            "floor": state.floor.to_string(),
            "direction": state.direction.to_string(),
            "cabRequests": cab_requests[id],
        });
    
        states_map.insert(id.to_string(), state_variables);
    }
    
    let json_input = serde_json::json!({
        "hallRequests": hall_requests,
        "states": states_map,
    });
    let json_input_string = json_input.to_string();

    let executable_path = "src/cost_function/executables/hall_request_assigner";

    let output = process::Command::new(executable_path)
        .args(&["-i", &json_input_string, "--includeCab"])
        .stdout(process::Stdio::piped())
        .spawn()
        .expect("Failed to start process")
        .wait_with_output()
        .expect("Failed to read stdout");

    let all_assigned_orders_string = String::from_utf8_lossy(&output.stdout).to_string();
    
    all_assigned_orders_string
}

