#include <errno.h>
#include <stdio.h>
#include <sys/personality.h>
#include <sys/syscall.h>
#include <unistd.h>

int main() {
  printf("Starting Seccomp test\n");

  printf("[*] syscall personality()...\n");
  int ret = syscall(SYS_personality, 0xffffffff);

  if (ret == -1 && errno == EPERM) {
    printf("SUCCESS: personality() has been blocked (EPERM).\n");
  } else if (ret >= 0) {
    printf("FAILURE: personality() succeeded (Filter is not working).\n");
  } else {
    printf("FAILURE: personality() failed with errno %d\n", errno);
  }

  printf("[*] syscall pivot_root()...\n");
  ret = syscall(SYS_pivot_root, ".", ".");

  if (ret == -1 && errno == EPERM) {
    printf("SUCCESS: pivot_root() has been blocked (EPERM).\n");
  } else if (ret >= 0) {
    printf("FAILURE: pivot_root() succeeded (This is bad!).\n");
  } else {
    printf("FAILURE: pivot_root() was not blocked by Seccomp (errno: %d, "
           "expected EPERM=1).\n",
           errno);
  }

  return 0;
}

