#[derive(Debug)]
pub struct ElevatorVariables {
    pub currentFloor: u8,
    pub direction: u8,
    pub state: State, //enum satt til idle, stopped, moving
}