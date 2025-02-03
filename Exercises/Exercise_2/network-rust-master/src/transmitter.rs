use std::env;
use std::process;
use std::thread::sleep;
use std::time::Duration;

use crossbeam_channel as cbc;
use network_rust::udpnet;

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
struct CustomDataType {
    message: String,
    iteration: u64,
}

fn main() -> std::io::Result<()> {
    let args: Vec<String> = env::args().collect();
    let id = if args.len() > 1 {
        args[1].clone()
    } else {
        format!("Sender#{}", process::id())
    };

    let msg_port = 19745;

    let (custom_data_send_tx, custom_data_send_rx) = cbc::unbounded::<CustomDataType>();
    // Periodisk sending av data
    std::thread::spawn(move || {
        let mut cd = CustomDataType {
            message: format!("Hello from {}", id),
            iteration: 0,
        };
        loop {
            custom_data_send_tx.send(cd.clone()).unwrap();
            cd.iteration += 1;
            sleep(Duration::new(1, 0));
        }
    });

    // Sender data på nettverket
    udpnet::bcast::tx(msg_port, custom_data_send_rx);
    

    Ok(())
}