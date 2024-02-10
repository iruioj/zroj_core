#ifndef SANDBOX_SIO_H
#define SANDBOX_SIO_H

/** This header defines async-signal-safe IO utilities */
#include <sys/types.h>

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

#endif