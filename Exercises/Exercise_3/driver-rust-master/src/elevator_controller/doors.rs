use crate::elevio::elev as e;
use crossbeam_channel as cbc;

pub fn door(
    elevator: e::Elevator,
    door_open_rx: cbc::Receiver<bool>,
    door_close_rx: cbc::Receiver<bool>,
    obstruction_rx: cbc::Receiver<bool>,
    obstructed_tx: cbc::Sender<bool>
) {

    elevator.door_light(false);
    loop {
        let mut obstructed: bool = false;
        cbc::select! {
            recv(obstruction_rx) -> a => {
                obstructed_tx.send(a.unwrap()).unwrap();
            },
            recv(door_open_rx) -> a => {

            },
            recv(door_close_rx) -> a => {

            }
        }
    } 

    

}