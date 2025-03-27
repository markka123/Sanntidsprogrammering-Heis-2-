# Elevator project in TTK4145

This program is an implementation of a scalable, real-time elevator system in rust.

To run the elevators:
1. Set the parameters in config/config.rs to match your setup.
2. Start an elevatorserver with the command ```elevatorserver --port <port_number>```
4. Run the program with the command ```run port=<port_number> id=<elevator_id>```

The id has to be unique, and between 0 and (config::ELEV_NUM_ELEVATORS-1). 
It is recomended to start the first elevator with id=0.

