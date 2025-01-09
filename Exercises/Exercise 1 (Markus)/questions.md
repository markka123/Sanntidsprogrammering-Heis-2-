Exercise 1 - Theory questions
-----------------------------

### Concepts

What is the difference between *concurrency* and *parallelism*?
> Concurrency incolves logical structuring of code so that multiple tasks can be handeled simultaniously (conceptionally!). Parallelism refers to a processor optimization technique that focuses on executing multiple instructions/tasks simultaniously. Thus, the main difference is that concurrency is a conceptional structuring of code so that tasks are dealt with simultainiously, while paralellism refers to the "physical" execution of multiple tasks at the same time.

What is the difference between a *race condition* and a *data race*? 
> A race condition happensm when two or more threads interact in an unpredictable manner. A data-race is a specific type of race condition where two or more threads access the same memory location concurrently and at least one of the accsesses is a write. My research for this question suggests that the issues experienced in task 3 are because of a data race. 
 
*Very* roughly - what does a *scheduler* do, and how does it do it?
> A scheduler decides how and when threads are executed by the cpu. It tracks all the threads that need to be executed and chooses which one should be executed next based on its scheduling policy. It does so through interrupts, queues and switching (saving the current state and jumping to another).


### Engineering

Why would we use multiple threads? What kinds of problems do threads solve?
> It allows our program to both conceptionally and actually solve multipe problems at the same time (concurrency and paralellism). It therefore improves program performance and is used in a variaty of applications such as real-time systems, I/O operations and processing of large datasets. 

Some languages support "fibers" (sometimes called "green threads") or "coroutines"? What are they, and why would we rather use them over threads?
> Fibers are user managed units of execution that allow programs to handle concurrency without relying on the operating system's thread management. These allow the programmer to tailor the running of seemingly paralell threads to the specific program. (Coroutines are actually run in the same thread.)

Does creating concurrent programs make the programmer's life easier? Harder? Maybe both?
> Concurrent programs can make the programmer's life easier on one side by allowing improved performance and simpler problem decomposition. On the other side it increases the complexity of coordination which make the programmers life harder. 

What do you think is best - *shared variables* or *message passing*?
> I believe that the answer to this question depends on the task at hand. If the tasks are tightly coupled and latency is critical shared variable appear more suitable, but they are at the risk of race conditions. Message passing avoid this issue, but might not achieve the same performance as shared variable systems. 


