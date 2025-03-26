use std::env;

pub fn fetch_command_line_arguments() -> (u16, u8) {
    let default_port = 15657;
    let default_elevator_id = 0;

    let mut port = default_port;
    let mut elevator_id = default_elevator_id;

    for arguments in env::args().skip(1) {
        if let Some(value) = arguments.strip_prefix("port=") {
            if let Ok(parsed_port) = value.parse::<u16>() {
                port = parsed_port;
            }
        } else if let Some(value) = arguments.strip_prefix("id=") {
            if let Ok(parsed_id) = value.parse::<u8>() {
                elevator_id = parsed_id;
            }
        }
    }
    
    (port, elevator_id)
}