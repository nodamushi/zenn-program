
#include <nuttx/config.h>
#include <pthread.h>
#include <sched.h>
#include <stdio.h>

typedef struct {
  volatile int *thread1;
  volatile int *thread3;
} state_t;

void *thread_1(void *arg) {
  volatile int *ptr = ((state_t *)arg)->thread1;
  for (int i = 0; i < 100000; i++) {
    *ptr = *ptr + 1;
  }
  return NULL;
}

void *thread_2(void *arg) {
  volatile int *ptr1 = ((state_t *)arg)->thread1;
  volatile int *ptr3 = ((state_t *)arg)->thread3;
  for (int i = 0; i < 30; i++) {
    int v1 = *ptr1;
    int v3 = *ptr3;
    printf("[Thread2] Thread1=%d, Thread3=%d\n", v1, v3);
  }
  printf("Thread 2 Done\n");
  return NULL;
}

void *thread_3(void *arg) {
  volatile int *ptr = ((state_t *)arg)->thread3;
  for (int i = 0; i < 100000; i++) {
    *ptr = *ptr + 1;
  }
  return NULL;
}

pthread_t run(int thread_id, pthread_startroutine_t f, state_t *arg) {

  struct sched_param sparam = {};
  switch (thread_id) {
  case 1:
    sparam.sched_priority = 110;
    break;
  case 2:
    sparam.sched_priority = 105;
    break;
  case 3:
    sparam.sched_priority = 100;
    break;
  }

  pthread_attr_t attr;
  pthread_attr_init(&attr);
  pthread_attr_setschedparam(&attr, &sparam);

  pthread_t p;
  pthread_create(&p, &attr, f, (void *)arg);

  return p;
}

int main(int argc, FAR char *argv[]) {
  volatile int t1 = 0;
  volatile int t3 = 0;
  state_t x = {&t1, &t3};
  printf("Start Threads\n");
  pthread_t p1 = run(1, thread_1, &x);
  pthread_t p2 = run(2, thread_2, &x);
  pthread_t p3 = run(3, thread_3, &x);

  pthread_join(p1, NULL);
  pthread_join(p2, NULL);
  pthread_join(p3, NULL);
  printf("Done\n");
  return 0;
}
