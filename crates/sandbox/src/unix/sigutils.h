#ifndef SANDBOX_SIGUTILS_H
#define SANDBOX_SIGUTILS_H

#include "sio.h"
#include <errno.h>
#include <signal.h>
#include <unistd.h>
#include <sys/wait.h>
#include <sys/resource.h>

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
void * signal_echo(int signo);

int get_sigchld();
int get_sigkill();
int get_sigxcpu();

#endif