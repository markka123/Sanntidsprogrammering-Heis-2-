# Peer review - architecture overview and design choices

### Design choices
The system is implemented as a UDP-based master-slave network. Every node can act as a master and is paired with a unique backup. Each elevator is identical, and resembles the following structure:
![Alt text](Modules.png)
(Note that this is an old scetch and that module-names and internal variables has changed. All TCP communication has been replaced by UDP broadcasting).

During normal mode of operation all elevators continously receives all elevators State objects.New-order and order-completed is transmitted when they occur. All elevators store these in respectively the States and allOrders objects, ensuring that all elevators have the information they need to step in as master at any time. The master acts as any other elevator, but is also responsible for assigning orders to each elevator based on the information in States and allOrders. When running the program each elevator receives its id as a command-line argument. The first elevator is assigned id=0, second elevator id=1 and so on. 


### Module Architecture & Threads
The proposed structure consists of the following modules:

Distributor
- receiver.rs:
A dedicated thread containing logic to receive UDP messages, sending these over appropriate channels and taking over as master if the current master fails and your ID is the next in line (elev 1 is backup for 0, 2 is backup for 1 ... and 0 is backup for the last elevator with id NUM_ELEVATORS-1).
- transmitter.rs:
A dedicated thread containing logic for broadcasting messages.
- distributor.rs:
Connecting the transmitter, receiver and distribution logic together, interpreting information received at the receiver, preparing messages to be sendt in the transmitter and sending assigned orders to the elevator_fsm module.

Elevator_controller
- direction.rs:
Contains the functions nessecary for altering and retrieving the direction of the elevator.
- doors.rs:
Contains the functions used to handle the elevators doors.
- lights.rs:
Contains the function responsible for setting the elevators lights.
- orders.rs:
Defines nessecary types and implements the AllOrders struct.
- elevator_fsm.rs:
The logic of a single elevator implemented as a statemachine. It runs the elevator based on a matrix containing the orders that the has been assigned.

Cost_function
- executables:
Contains the handed out executables used to assign orders efficiently.
- cost_function.rs:
Contains a function that structure the input parameters, calls the executable and returns the a list of the assigned orders on a JSON format.

Network
- udp.rs:
Contains basic functions for UDP transmission.

Config
- config.rs:
Contains global constants that are known at compile-time.

Elevio: Handed out module responsible for communication with elevator I/O
- elev.rs
- poll.rs




