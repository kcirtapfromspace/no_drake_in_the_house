#include <stdio.h>
#include <unistd.h>

int main() {
    printf("Hello from C program!\n");
    fflush(stdout);
    sleep(5);
    printf("Still running...\n");
    fflush(stdout);
    return 0;
}