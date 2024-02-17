#include <errno.h>
#include <signal.h>
#include <sys/resource.h>
#include <sys/types.h>
#include <sys/wait.h>
#include <unistd.h>

#ifndef SANDBOX_UTILS_H
#define SANDBOX_UTILS_H

/* Private sio_functions */
/* sio_reverse - Reverse a string (from K&R) */
void sio_reverse(char *const s);

/* sio_ltoa - Convert long to base b string (from K&R) */
void sio_ltoa(long v, char *const s, int b);

/* sio_strlen - Return length of string (from K&R) */
size_t sio_strlen(const char *const s);

/* Public Sio functions */
ssize_t sio_dputs(int fd, const char *const s);

/* Put long */
ssize_t sio_dputl(int fd, long v);

/* Put error message and exit */
void sio_error(const char *const s);

typedef struct {
  /* Total amount of user time used.  */
  struct timeval ru_utime;
  /* Total amount of system time used.  */
  struct timeval ru_stime;
  /* Maximum resident set size (in kilobytes).  */
  long int ru_maxrss;
} rusage_t;

int get_children_rusage(rusage_t *ru);
int get_self_rusage(rusage_t *ru);

typedef struct {
  rusage_t rusage;
  int timer_first;
  int status;
} global_shared_t;

global_shared_t *init_shared();
void free_shared(global_shared_t *);

int get_errno();

sigset_t sigblockall();
sigset_t Sigsetmask(sigset_t mask);
int Setrlimit(int resource, rlim_t rlim_cur, rlim_t rlim_max);

int open_read_file(const char *filename);
int open_write_file(const char *filename);

int wrap_WIFEXITED(int status);
int wrap_WIFSIGNALED(int status);
int wrap_WEXITSTATUS(int status);
int wrap_WTERMSIG(int status);

// a echo handler for signal (for debugging)
void *signal_echo(int signo);

int get_sigchld();
int get_sigkill();
int get_sigxcpu();

int wait_rusage(pid_t pid, int *stat_loc, int options, rusage_t *ru);

#endif