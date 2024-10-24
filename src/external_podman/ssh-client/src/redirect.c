// connect() fook lib
//
// gcc -shared -fPIC redirect.c -o redirect.so -ldl
// export LD_PRELOAD=/path/to/redirect.so
// export REDIRECT_ADDRS="from:to,from2:to2,from3:to3"
//
#ifndef MAX_MAPPINGS
#  define MAX_MAPPINGS 8
#endif

#ifndef ENV_NAME
#  define ENV_NAME "REDIRECT_ADDRS"
#endif


#define _GNU_SOURCE
#include <arpa/inet.h>
#include <dlfcn.h>
#include <netinet/in.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/socket.h>

struct addr_mapping {
  in_addr_t from;
  in_addr_t to;
};

static struct addr_mapping mappings[MAX_MAPPINGS];
static int mapping_count = 0;

__attribute__((constructor)) static void init(void) {
  mapping_count = 0;
  char *redirect_conf = getenv(ENV_NAME);
  if (!redirect_conf)
    return;

  char *conf_copy = strdup(redirect_conf);
  char *pair = strtok(conf_copy, ",");

  while (pair && mapping_count < MAX_MAPPINGS) {
    char *from_str = strtok(pair, ":");
    char *to_str = strtok(NULL, ":");

    if (from_str && to_str) {
      mappings[mapping_count].from = inet_addr(from_str);
      mappings[mapping_count].to = inet_addr(to_str);
      mapping_count++;
    }
    pair = strtok(NULL, ",");
  }
  free(conf_copy);
}

int connect(int sockfd, const struct sockaddr *addr, socklen_t addrlen) {
  typedef int (*fun)(int, const struct sockaddr *, socklen_t);
  fun real_connect = dlsym(RTLD_NEXT, "connect");

  if (mapping_count && addr->sa_family == AF_INET) {
    struct sockaddr_in *addr_in = (struct sockaddr_in *)addr;
    for (int i = 0; i < mapping_count; i++) {
      if (addr_in->sin_addr.s_addr == mappings[i].from) {
        addr_in->sin_addr.s_addr = mappings[i].to;
        break;
      }
    }
  }
  return real_connect(sockfd, addr, addrlen);
}