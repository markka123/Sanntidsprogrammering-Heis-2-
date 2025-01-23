// Compile with `gcc foo.c -Wall -std=gnu99 -lpthread`, or use the makefile
// The executable will be named `foo` if you use the makefile, or `a.out` if you use gcc directly

#include <pthread.h>
#include <stdio.h>

int i = 0;
pthread_mutex_t mutex;

// Note the return type: void*
void* incrementingThreadFunction(){
    // TODO: increment i 1_000_000 times  
    for(int j = 0; j < 1000000; j++) {
        pthread_mutex_lock(&mutex);
        i++;
        pthread_mutex_unlock(&mutex);
    }
    return NULL;
}

void* decrementingThreadFunction(){
    // TODO: decrement i 1_000_000 times    
    for(int j = 0; j < 999999; j++) {
        pthread_mutex_lock(&mutex);
        i--;
        pthread_mutex_unlock(&mutex);
    }
    return NULL;
}



int main(){
    // TODO: 
    // start the two functions as their own threads using `pthread_create`
    // Hint: search the web! Maybe try "pthread_create example"?
    pthread_mutex_init(&mutex, NULL);

    pthread_t thread_id_inc;
    pthread_t thread_id_dec;
    pthread_create(&thread_id_inc, NULL, incrementingThreadFunction, NULL);
    pthread_create(&thread_id_dec, NULL, decrementingThreadFunction, NULL);
    // TODO:
    // wait for the two threads to be done before printing the final result
    // Hint: Use `pthread_join`    
    pthread_join(thread_id_inc, NULL);
    pthread_join(thread_id_dec, NULL);
    
    pthread_mutex_destroy(&mutex);
    printf("The magic number is: %d\n", i);

    return 0;
}
