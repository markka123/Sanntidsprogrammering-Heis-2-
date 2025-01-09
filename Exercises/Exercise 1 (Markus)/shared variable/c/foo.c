// Compile with `gcc foo.c -Wall -std=gnu99 -lpthread`, or use the makefile
// The executable will be named `foo` if you use the makefile, or `a.out` if you use gcc directly

#include <pthread.h>
#include <stdio.h>

int i = 0;
pthread_mutex_t mutex;

void* incrementing() {

    // Add mutex to avoid race conditions

    for (int j = 0; j < 1000000; j++) {
        pthread_mutex_lock(&mutex);
        i++;
        pthread_mutex_unlock(&mutex);
    }

    return NULL;
}

void* decrementing() { 
    
    //Add mutex to avoid race conditions

    for (int j = 0; j < 1000001; j++) {
        pthread_mutex_lock(&mutex);
        i--;
        pthread_mutex_unlock(&mutex);
    }

    return NULL;
}

int main() {
    // Initialize the two threads
    pthread_t incrementingThread, decrementingThread; 

    // Initialize the mutex
    pthread_mutex_init(&mutex, NULL); 

    pthread_create(&incrementingThread, NULL, incrementing, NULL);
    pthread_create(&decrementingThread, NULL, decrementing, NULL);

    // Makes sure that the threads finish before proceeding
    pthread_join(incrementingThread, NULL); 
    pthread_join(decrementingThread, NULL);

    //Clean up the mutex
    pthread_mutex_destroy(&mutex);

    printf("The magic number is: %d\n", i);
    return 0;
}