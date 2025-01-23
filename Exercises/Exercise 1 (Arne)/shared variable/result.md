3.
In C: the result seems to be random. It is random because of race conditions. Since both can read it before either writes it back, one update can owerwrite the other, leading to lost increments or decrements.  
In GO: GOMAXPROCS set to 1 will give 0. GOMAXPROCS set to 2 will give random result. GOMAXPROCS is how many operating systems threads that Go can use to execute gorutines. Setting it to 2 leads to race conditions and therfore the random behaviour in the result. 

4.
In C: Mutex should be used since it can enforce exclusive access to a resource. Semaphore is used when multiple threads should work simultaneously. 

In GO: Using channels and a server gave the right outcome

5.

