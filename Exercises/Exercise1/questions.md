Exercise 1 - Theory questions
-----------------------------

### Concepts

What is the difference between *concurrency* and *parallelism*?
> *Your answer here*
Concurrency handle multiple tasks at the same time by interleaving their execution. The goal is to make progress without blocking or wating unnecessarily, achived by task switching. Tasks can make progress simuntaneously, even if they dont run at the same time.

Parallelism involves executing multiple tasks simultaneously, typically on multiple cores. 


What is the difference between a *race condition* and a *data race*? 
> *Your answer here* 
Race conditions are where the system's substantive beahvior is dependent on the timing or sequence of events. 

Data races occurs when two or more threads in a single process access the same memory location concurrently, and one of the accesses is for writing, and threads are not using locks. 


*Very* roughly - what does a *scheduler* do, and how does it do it?
> *Your answer here* 
Action of assigning resources to perform tasks. Tasks may be threads, processes or data flows. 


### Engineering

Why would we use multiple threads? What kinds of problems do threads solve?
> *Your answer here*
Threads allow tasks to run concurrently, and also parallelism. It solves multitasking, data processing pipelines, handle multiple I/O streams and more.  

Some languages support "fibers" (sometimes called "green threads") or "coroutines"? What are they, and why would we rather use them over threads?
> *Your answer here*
A thread that is scheduled by a runtime library or VM, instead of the underlying OS. Managed in user space instead of kernel space

Switching between threads is faster, cosnume less memory and CPU overhead

Does creating concurrent programs make the programmer's life easier? Harder? Maybe both?
> *Your answer here*
Easier:
improved performance (handle multiple tasks), better responsivness even with slow  I/O in the background, can simplify code for handling asynchronous operations. 

Harder:
Increased complexity, non-deterministic behaviour. Ex race conditions that can make bugs. Harder to debug and test, can change depending on thread scheduling and timing. 

What do you think is best - *shared variables* or *message passing*?
> *Your answer here*
I think massage passing is best. No need for locks or other synchronization primitives. No risk for race conditions and easier to scale. 


