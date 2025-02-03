use std::env;
use std::process;
use std::thread::sleep;
use std::time::Duration;

use crossbeam_channel as cbc;
use network_rust::udpnet;
use std::net::UdpSocket;

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

    let msg_port = 20003;
    let target_ip = "10.100.23.204";
    let target_address = format!("{}:{}", target_ip, msg_port);

    let socket = UdpSocket::bind("0.0.0.0:0")?;

    let (custom_data_send_tx, custom_data_send_rx) = cbc::unbounded::<CustomDataType>();
    // Periodisk sending av data
    std::thread::spawn(move || {
        let mut cd = CustomDataType {
            message: format!("Hallaballa {}", id),
            iteration: 0,
        };
        loop {
            custom_data_send_tx.send(cd.clone()).unwrap();
            cd.iteration += 1;
            sleep(Duration::new(1, 0));
        }
    });

    // Sender data på nettverket
    //udpnet::bcast::tx(msg_port, custom_data_send_rx);
    for data in custom_data_send_rx.iter() {
        let serialized_data = serde_json::to_string(&data).unwrap();
        socket.send_to(serialized_data.as_bytes(), &target_address)?;
        println!("Sent message: {}", serialized_data);
    }
    

    Ok(())
}