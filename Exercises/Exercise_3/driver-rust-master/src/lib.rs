// src/lib.rs
pub mod elevio {
    pub mod elev;
    pub mod poll;
}

pub mod elevator_controller {
    pub mod order_handler;
    pub mod state_machine;
}

pub mod button_handler {
    pub mod create_order;
}