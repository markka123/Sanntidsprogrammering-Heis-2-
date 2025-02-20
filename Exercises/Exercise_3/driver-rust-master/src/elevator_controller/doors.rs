use crate::elevio::elev as e;
use crate::config::config;
use crossbeam_channel as cbc;

pub fn door(
    elevator: e::Elevator,
    door_open_rx: &cbc::Receiver<bool>,
    door_close_tx: &cbc::Sender<bool>,
    obstruction_rx: &cbc::Receiver<bool>,
    obstructed_tx: &cbc::Sender<bool>
) {

    elevator.door_light(false);

    let mut is_obstructed = false;
    let mut is_door_open = false;
    let mut door_timer = cbc::never();

    loop {
        cbc::select! {
            recv(door_open_rx) -> _ => {
                is_door_open = true;
                elevator.door_light(true);
                if is_obstructed {
                    door_timer = cbc::never();
                }
                else {
                    door_timer = cbc::after(config::DOOR_TIMER_DURATION);   
                }
            },
            recv(obstruction_rx) -> a => {
                let obstruction_status = a.unwrap();
                obstructed_tx.send(obstruction_status).unwrap();

                is_obstructed = obstruction_status;

                if is_obstructed {
                    door_timer = cbc::never();
                } else if is_door_open {
                    door_timer = cbc::after(config::DOOR_TIMER_DURATION);
                }
            },

            recv(door_timer) -> _ => {
                is_door_open = false;
                elevator.door_light(false);
                door_close_tx.send(true).unwrap();
            }
        }
    }
}