// REASSIGNING ALL ORDERS

// INPUT
// {
//     "hallRequests" : 
//         [[Boolean, Boolean], ...],
//     "states" : 
//         {
//             "id_1" : {
                    
//                 "behaviour"     : < "idle" | "moving" | "doorOpen" >
//                 "floor"         : NonNegativeInteger
//                 "direction"     : < "up" | "down" | "stop" >
//                 "cabRequests"   : [Boolean, ...]
//             },
//             "id_2" : {...}
//         }
// }

//Til ANDREAS: Jeg vil ha states som en matrise der hver heis har sin rad (id1 -> rad 1) og de relevante variablene ligger på den raden i rekkefølgen [behaviour, floor, direction], de mulige alternativene står over og her er et eksempel:   let elevator_variables = vec![vec!["moving".to_string(), "2".to_string(), "up".to_string()]];

#![allow(dead_code)]
// PACKAGES
use crate::elevator_controller::orders::{CabOrders, HallOrders};
use crate::distributor::distributor::{States};
use crate::elevator_controller::elevator_fsm;
use crate::elevator_controller::direction;

use std::process::{Command, Stdio};
use serde_json::{json};
use num2words::Num2Words;

//ASSIGN_REQUESTS
pub fn assign_orders(
    states: &States,
    cab_requests: &CabOrders,
    hall_requests: &HallOrders,
) -> String{

    let mut states_map = serde_json::Map::new();

    for (id, state) in states.iter().enumerate() {
        if state.motorstop || state.emergency_stop || state.obstructed || state.offline {
            continue;
        }
        let state_variables = json!({
            "behaviour": elevator_fsm::behaviour_to_string(state.behaviour),
            "floor": state.floor.to_string(),
            "direction": direction::direction_to_string(state.direction), // fix when an elevator should have dir stop
            "cabRequests": cab_requests[id],
        });
    
        states_map.insert(id.to_string(), state_variables);
    }
    
    let json_input = json!({
        "hallRequests": hall_requests,
        "states": states_map,
    });

    let json_input_string = json_input.to_string();
    // //println!("JSON Input:\n{}", json_input_string);

    // Path to the executable
    let executable_path = "src/cost_function/executables/hall_request_assigner";

    // Call the executable using command-line arguments
    let output = Command::new(executable_path)
        .args(&["-i", &json_input_string, "--includeCab"])
        .stdout(Stdio::piped())
        .spawn()
        .expect("Failed to start process")
        .wait_with_output()
        .expect("Failed to read stdout");

    let assigned_orders_str = String::from_utf8_lossy(&output.stdout).to_string();
    
    assigned_orders_str
}

