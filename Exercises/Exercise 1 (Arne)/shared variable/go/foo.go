// Use `go run foo.go` to run your program

package main

import (
    . "fmt"
    "runtime"
    "time"
)



func incrementing(incrementCh, doneCh chan struct{}) {
    //TODO: increment i 1000000 times
    for j:=0; j < 1000000; j++ {
        incrementCh <- struct{}{}
    }
    doneCh <- struct{}{}

}

func decrementing(decrementCh, doneCh chan struct{}) {
    //TODO: decrement i 1000000 times
    for j:=0; j < 999999; j++ {
        decrementCh <- struct{}{}
    }
    doneCh <- struct{}{}
}

func numberServer(incrementCh, decrementCh, doneCh chan struct{}, resultCh chan int) {
    var i int
    var doneCount int
    doneCount = 0

    for {
        select {
        case <- incrementCh:
            i++
        case <- decrementCh:
            i--
        case <- doneCh:
            doneCount ++
            if doneCount == 2 {
                resultCh <- i
                close(resultCh)
                return
            }
        }
    }
}

func main() {
    
    // What does GOMAXPROCS do? What happens if you set it to 1?
    runtime.GOMAXPROCS(2)    
	
    // TODO: Spawn both functions as goroutines

    incrementCh := make(chan struct{})
    decrementCh := make(chan struct{})
    doneCh := make(chan struct{})
    resultCh := make(chan int)

    go numberServer(incrementCh, decrementCh, doneCh, resultCh)
    go incrementing(incrementCh, doneCh)
    go decrementing(decrementCh, doneCh)

    finalValue := <- resultCh
	
    // We have no direct way to wait for the completion of a goroutine (without additional synchronization of some sort)
    // We will do it properly with channels soon. For now: Sleep.
    time.Sleep(500*time.Millisecond)
    Println("The magic number is:", finalValue)
}
