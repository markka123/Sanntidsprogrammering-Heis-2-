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

//ANDREAS: Jeg vil ha states som en matrise der hver heis har sin rad (id1 -> rad 1) og de relevante variablene ligger på den raden i rekkefølgen [behaviour, floor, direction], de mulige alternativene står over og her er et eksempel:   let elevator_variables = vec![vec!["moving".to_string(), "2".to_string(), "up".to_string()]];


// PACKAGES
use std::process::{Command, Stdio};
use serde_json::{json};
use num2words::Num2Words;



//ASSIGN_REQUESTS
fn assign_requests(
    elevator_variables: Vec<Vec<String>>,
    cab_requests: Vec<Vec<bool>>,
    hall_requests: Vec<Vec<bool>>,
) -> String {

    let mut states = serde_json::Map::new();

    for (i, elevator) in elevator_variables.iter().enumerate() {
        let state = json!({
            "behaviour": elevator[0],
            "floor": elevator[1].parse::<i32>().unwrap(),
            "direction": elevator[2],
            "cabRequests": cab_requests[i],
        });
    
        let word = Num2Words::new(i as u32 + 1)
            .to_words()
            .unwrap_or_else(|_| (i + 1).to_string()); // Handle errors safely
    
        states.insert(word, state);
    }
    

    let json_input = json!({
        "hallRequests": hall_requests,
        "states": states,
    });

    let json_input_string = json_input.to_string();
    println!("JSON Input:\n{}", json_input_string);

    // Path to the executable
    let executable_path = "src/offlineOrderHandler/executables/hall_request_assigner.exe";

    // Call the executable using command-line arguments
    let output = Command::new(executable_path)
        .args(&["-i", &json_input_string, "--includeCab"])
        .stdout(Stdio::piped())
        .spawn()
        .expect("Failed to start process")
        .wait_with_output()
        .expect("Failed to read stdout");

    let output_str = String::from_utf8_lossy(&output.stdout).to_string();
    output_str
}

//MAIN

fn main() {
    let elevator_variables = vec![vec!["moving".to_string(), "2".to_string(), "up".to_string()]];
    let cab_requests = vec![vec![false, false, true, true]];
    let hall_requests = vec![vec![false, false], vec![true, false], vec![false, false], vec![false, true]];


    let result = assign_requests(elevator_variables, cab_requests, hall_requests);
    println!("Result from executable:\n{}", result);

}



