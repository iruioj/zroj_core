#include "checker_c_abi.h"
#include <stdio.h>
#include <unistd.h>

int main(int argc, char **argv) {
  if (argc != 2) {
    fprintf(stderr, "invalid argument count, expect 1 argument\n");
    return 1;
  }

  int r = chdir(argv[1]);
  if (r != 0) {
    fprintf(stderr, "failed to change working directory, path = \"%s\"",
            argv[1]);
    return 1;
  }

  float score = check();

  printf("\n%.6lf", score);
  return 0;
}