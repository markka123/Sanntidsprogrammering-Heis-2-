// Use `go run foo.go` to run your program

package main

import (
    . "fmt"
    "runtime"
    "sync"
    "time"
)

var (
    i     = 0
    mutex sync.Mutex //Initialize the mutex
)

func incrementing() {

    // Add mutex to avoid race conditions

    for j := 0; j < 1000000; j++ {
        mutex.Lock()
        i++
        mutex.Unlock()
    }
}

func decrementing() {

    // Add mutex to avoid race conditions

    for j := 0; j < 1000001; j++ {
        mutex.Lock()
        i--
        mutex.Unlock()
    }
}

func main() {

    // Allow go runtime to use up to two threads for executing go-routines
    runtime.GOMAXPROCS(2)


    // Executing the go-routines
    go incrementing()
    go decrementing()

    //Pauses the main-function to give the go-routines enough time to finish
    time.Sleep(500 * time.Millisecond)
    Println("The magic number is:", i)
}