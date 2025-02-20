
fn transmitter() {
    let state;
        loop {
        select: 
            recv call_btn_rx {
                if hall {
                    send on hall udp
                } if cab {
                    send on cab udp
                }
            },
            recv delivered_order_rx {
                if hall_finish {
                    send on hall_finish udp
                } if cab_finish {
                    send on cab_finish udp
                }
            }
            recv new_state_rx => a {
                state = a.unwrap()
            }
        send state }
    
    thread master_transmitter(master_rx)
}


fn master_transmitter() {
    loop {
        select:
            
    }
}