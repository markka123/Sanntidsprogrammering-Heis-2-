use std::process;

use crossbeam_channel as cbc;
use network_rust::udpnet;

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
struct CustomDataType {
    message: String,
    iteration: u64,
}

fn main() -> std::io::Result<()> {
    let msg_port = 30000;

    let (custom_data_recv_tx, custom_data_recv_rx) = cbc::unbounded::<CustomDataType>();
    // Mottar data på nettverket
    std::thread::spawn(move || {
        udpnet::bcast::rx(msg_port, custom_data_recv_tx)
    });
        

    // Hovudløkka for å ta imot data
    loop {
        cbc::select! {
            recv(custom_data_recv_rx) -> data => {
                match data {
                    Ok(cd) => {
                        println!("Received: {:#?}", cd);
                    }
                    Err(e) => {
                        eprintln!("Error receiving data: {}", e);
                    }
                }
            }
        }
    }
}
