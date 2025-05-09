#pragma once

#include <stdint.h>

struct SubsystemDescription;

typedef int (*init_subsystem)(struct SubsystemDescription *);

typedef struct SubsystemDescription {
  const char *name;
  const char *brief;
  const char *author;
  init_subsystem init;
} SubsystemDescription;

#define KHZEB_SUBSYSTEM(n, b)                                                  \
  static const SubsystemDescription subsys = {.name = n, .brief = b};          \
  const SubsystemDescription *get_subsystem() { return &subsys; }\
