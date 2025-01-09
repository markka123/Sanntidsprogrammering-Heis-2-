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
    mutex sync.Mutex
)

func incrementing() {
    for j := 0; j < 1000001; j++ {
        mutex.Lock()
        i++
        mutex.Unlock()
    }
}

func decrementing() {
    for j := 0; j < 1000000; j++ {
        mutex.Lock()
        i--
        mutex.Unlock()
    }
}

func main() {
    runtime.GOMAXPROCS(2)

    go incrementing()
    go decrementing()

    time.Sleep(500 * time.Millisecond)
    Println("The magic number is:", i)
}