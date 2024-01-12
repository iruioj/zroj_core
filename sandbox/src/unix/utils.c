#include "share.h"
#include "sigutils.h"
#include <asm-generic/errno-base.h>
#include <fcntl.h>
#include <signal.h>
#include <sys/mman.h>
#include <sys/resource.h>
#include <unistd.h>

#define MAXLINE 1024 /* max line size */

int get_errno() { return errno; }

/* Private sio_functions */
/* sio_reverse - Reverse a string (from K&R) */
void sio_reverse(char *const s) {
  int c, i, j;

  for (i = 0, j = strlen(s) - 1; i < j; i++, j--) {
    c = s[i];
    s[i] = s[j];
    s[j] = c;
  }
}

/* sio_ltoa - Convert long to base b string (from K&R) */
void sio_ltoa(long v, char *const s, int b) {
  int c, i = 0;
  v = (unsigned long)v;
  do {
    s[i++] = ((c = (v % b)) < 10) ? c + '0' : c - 10 + 'a';
  } while ((v /= b) > 0);
  s[i] = '\0';
  sio_reverse(s);
}

/* sio_strlen - Return length of string (from K&R) */
size_t sio_strlen(const char *const s) {
  int i = 0;

  while (s[i] != '\0')
    ++i;
  return i;
}

/* sio_copy - Copy len chars from fmt to s (by Ding Rui) */
void sio_copy(char *s, const char *const fmt, size_t len) {
  if (!len)
    return;

  for (size_t i = 0; i < len; i++)
    s[i] = fmt[i];
}

/* Public Sio functions */

/* Put string */
ssize_t sio_dputs(int fd, const char *const s) {
  return write(fd, s, sio_strlen(s));
}

/* Put long */
ssize_t sio_dputl(int fd, long v) {
  char s[128];

  sio_ltoa(v, s, 10); /* Based on K&R itoa() */
  return sio_dputs(fd, s);
}

/* Put error message with errno and exit */
void sio_error(const char *const s) {
  sio_dputs(STDERR_FILENO, s);
  sio_dputs(STDERR_FILENO, " (errno = ");
  sio_dputl(STDERR_FILENO, errno);
  sio_dputs(STDERR_FILENO, ")\n");
  _exit(1);
}

/* Wrappers for syscalls */

sigset_t Sigsetmask(sigset_t mask) {
  sigset_t prev;
  if (sigprocmask(SIG_SETMASK, &mask, &prev) < 0)
    sio_error("sigprocmask error");
  return prev;
}

sigset_t sigblockall() {
  sigset_t mask;
  sigfillset(&mask);
  return Sigsetmask(mask);
}

int open_read_file(const char *filename) {
  int fd = open(filename, O_RDONLY);
  return fd;
}

int open_write_file(const char *filename) {
  int fd = open(filename, O_WRONLY | O_CREAT | O_TRUNC, 0644);
  return fd;
}

int Setrlimit(int resource, rlim_t rlim_cur, rlim_t rlim_max) {
  struct rlimit lim;
  lim.rlim_cur = rlim_cur;
  lim.rlim_max = rlim_max;

  return setrlimit(resource, &lim);
}

int wrap_WIFEXITED(int status) { return WIFEXITED(status); }
int wrap_WIFSIGNALED(int status) { return WIFSIGNALED(status); }
int wrap_WEXITSTATUS(int status) { return WEXITSTATUS(status); }
int wrap_WTERMSIG(int status) { return WTERMSIG(status); }

global_shared_t *init_shared() {
  global_shared_t *global_shared =
      mmap(NULL, sizeof *global_shared, PROT_READ | PROT_WRITE,
           MAP_SHARED | MAP_ANONYMOUS, -1, 0);
  if (global_shared == MAP_FAILED) {
    sio_error("mmap error");
  }
  return global_shared;
}

void free_shared(global_shared_t *global_shared) {
  if (munmap(global_shared, sizeof *global_shared) < 0)
    sio_error("munmap error");
  global_shared = NULL;
}
int get_children_rusage(rusage_t *ru) {
  struct rusage r;
  int rc = getrusage(RUSAGE_CHILDREN, &r);
  ru->ru_stime = r.ru_stime;
  ru->ru_utime = r.ru_utime;
  ru->ru_maxrss = r.ru_maxrss;
  return rc;
}

void signal_echo_handler(int signo) { psignal(signo, "receve signal"); }
void *signal_echo(int signo) { return signal(signo, signal_echo_handler); }