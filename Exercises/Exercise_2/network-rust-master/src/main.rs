use std::env;
use std::net;
use std::process;
use std::thread::*;
use std::time::Duration;

use crossbeam_channel as cbc;
use network_rust::udpnet;

// Data types to be sent on the network must derive traits for serialization
#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
struct CustomDataType {
    message: String,
    iteration: u64,
}

fn main() -> std::io::Result<()> {
    // Genreate id: either from command line, or a default rust@ip#pid
    let args: Vec<String> = env::args().collect();
    let id = if args.len() > 1 {
        args[1].clone()
    } else {
        let local_ip = net::TcpStream::connect("8.8.8.8:53")
            .unwrap()
            .local_addr()
            .unwrap()
            .ip();
        format!("Sendig msg from: rust@{}#{}", local_ip, process::id())
    };

    let msg_port = 19735;
    let peer_port = 20003; //19738
    let server_port = 30000;

    // The sender for peer discovery
    let (peer_tx_enable_tx, peer_tx_enable_rx) = cbc::unbounded::<bool>();
    let _handler = {
        let id = id.clone();
        spawn(move || {
            if udpnet::peers::tx(peer_port, id, peer_tx_enable_rx).is_err() {
                // crash program if creating the socket fails (`peers:tx` will always block if the
                // initialization succeeds)
                process::exit(1);
            }
        })
    };

    // (periodically disable/enable the peer broadcast, to provoke new peer / peer loss messages)
    // This is only for demonstration purposes, if using this module in your project do not include
    // this
    spawn(move || loop {
        sleep(Duration::new(6, 0));
        peer_tx_enable_tx.send(false).unwrap();
        sleep(Duration::new(3, 0));
        peer_tx_enable_tx.send(true).unwrap();
    });

    // The receiver for peer discovery updates
    let (peer_update_tx, peer_update_rx) = cbc::unbounded::<udpnet::peers::PeerUpdate>();
    {
        spawn(move || {
            if udpnet::peers::rx(peer_port, peer_update_tx).is_err() {
                // crash program if creating the socket fails (`peers:rx` will always block if the
                // initialization succeeds)
                process::exit(1);
            }
        });
    }

    // Periodically produce a custom data message
    let (custom_data_send_tx, custom_data_send_rx) = cbc::unbounded::<CustomDataType>();
    let (custom_data_send_tx, custom_rx) = cbc::unbounded::<CustomDataType>();
    
    
    {
        spawn(move || {
            let mut cd = CustomDataType {
                message: format!("Hello from node {}", id),
                iteration: 0,
            };
            loop {
                custom_data_send_tx.send(cd.clone()).unwrap();
                cd.iteration += 1;
                sleep(Duration::new(1, 0));
            }
        });
    }

    
    // The sender for our custom data
    {
        spawn(move || {
            if udpnet::bcast::tx(msg_port, custom_data_send_rx).is_err() {
                // crash program if creating the socket fails (`bcast:tx` will always block if the
                // initialization succeeds)
                process::exit(1);
            }
        });
    }
    // The receiver for our custom data
    let (custom_data_recv_tx, custom_data_recv_rx) = cbc::unbounded::<CustomDataType>();
    spawn(move || {
        if udpnet::bcast::rx(msg_port, custom_data_recv_tx).is_err() {
            // crash program if creating the socket fails (`bcast:rx` will always block if the
            // initialization succeeds)
            process::exit(1);
        }
    });


    // the receiver for server messages
    let (server_recv_tx, server_recv_rx) = cbc::unbounded::<String>();
    spawn(move || {
        if udpnet::bcast::rx(server_port, server_recv_tx).is_err() {
            // crash program if creating the socket fails (`bcast:rx` will always block if the
            // initialization succeeds)
            process::exit(1);
        }
    });


    // main body: receive peer updates and data from the network 
    loop {
        cbc::select! {
            recv(peer_update_rx) -> a => {
                let update = a.unwrap();
                println!("new peer identified: {:#?}", update);
            }
            recv(custom_data_recv_rx) -> a => {
                let cd = a.unwrap();
                println!("\n recieved custom msg: \n - message: {} \n - iteration: {} \n", cd.message, cd.iteration);
            }
            recv(server_recv_rx) -> a => {
                let cd = a.unwrap();
                println!("Recieved from server: {}", cd);
            }
        }
    }
}
