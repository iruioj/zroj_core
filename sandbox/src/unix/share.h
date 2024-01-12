/* memory sharing utils */

#include "sio.h"
#include <sys/resource.h>

typedef struct {
  /* Total amount of user time used.  */
  struct timeval ru_utime;
  /* Total amount of system time used.  */
  struct timeval ru_stime;
  /* Maximum resident set size (in kilobytes).  */
  long int ru_maxrss;
} rusage_t;

int get_children_rusage(rusage_t * ru);

typedef struct {
  rusage_t rusage;
  int timer_first;
  int status;
} global_shared_t;

global_shared_t *init_shared();
void free_shared(global_shared_t *);