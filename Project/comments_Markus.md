## Comments from review:



#### Should discuss:
Generelt:
- Writeout crossbeamchannel instad of cbc?

main:
- Synes ikke det er helt ryddig med den loopen og OK(()) på slutten -> burde se litt på det

elevator_fsm:
- motorstop_rx og tx er en channel som sender ting internt i en thread -> virker unødvendig? (Mistenker at vi kan droppe hele motorstop_rx recv-en og heller oppdatere staten inne i motor_timer og det andre stedet tx-en kalles)
- Vurdere å bake inn følgende kodelinje i en funksjon/variabel: elevator_orders.orders[state.floor as usize].iter().all(|&x| x == false)
- Er det nødvendig å initialisere emergency stop lyset? Vi setter det vel rett over til av også blir det oppdatert ved alle endringer?
- Skulle vi laget en init funksjon? (Kunne også vurdert å legge til en initialiseringsstate der det ikke blir assignet noen ordre)
- Kunne vurdert å skrive ut argumentene i recv casen istedenfor typ (new_order_tuple)
- Alle andre steder enn i order_done har vi sending på utsiden av ordren (order_done_tx)
- Vi er ikke helt konsekvente på hvor vi plasserer sending (tx-er), burde bestemme oss for hvor de skal være.
- Rakk ikke se grundig på emergency stop, men det virker som deler kanskje kan kortes ned eller pakkes inn i en funksjon?
- Synes vi generelt bør se på om fsm logikken er enkel nok å lese eller bør ha kommentarer
- Er det mulig å hoppe inn i denne casen på linje 190 ish? Kompilatoren bryr seg ikke om ; eller ikke 
                _ => {
                        println!("Floor indicator received while in unexpected state")
                    }

lights:
- Trenger vi denne linjen?         elevator.call_button_light(floor, elev::CAB, elevator_orders.orders[floor as usize][elev::CAB as usize]);

orders:
- Legge til let _ der vi sender på order_done_tx gjør at vi ignorerer erroren -> burde vi ikke gjøre dette alle andre ganger vi sender på en tx eller ikke i det hele tatt her? Evt generelt om vi skulle håndtert den erroren? (Klar over at å fjerne let _ vil introdusere en warning fra compilern)

