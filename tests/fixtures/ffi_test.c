#include <stdint.h>
#include <stdio.h>
#include <string.h>

int32_t add_ints(int32_t a, int32_t b) { return a + b; }

double add_doubles(double a, double b) { return a + b; }

int32_t strlen_c(const char *s) { return (int32_t)strlen(s); }

const char *greet(const char *name) {
  static char buf[256];
  snprintf(buf, sizeof(buf), "Hello, %s!", name);
  return buf;
}
