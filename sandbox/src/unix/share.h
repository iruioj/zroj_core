/* memory sharing utils */

#include "sio.h"
#include <sys/resource.h>

typedef struct {
  struct rusage rusage;
  int timer_first;
  int status;
} global_shared_t;

global_shared_t* init_shared();
void free_shared(global_shared_t *);