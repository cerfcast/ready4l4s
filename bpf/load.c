#include <stdio.h>
#include <sys/types.h>
#include <unistd.h>
#include <stdlib.h>
#include <net/if.h>

#include "bpf/libbpf.h"
#include "xdp/libxdp.h"
#include "bpf/bpf.h"

extern int errno;

int main() {

  int interface_id = if_nametoindex("tailscale0");
  if (!interface_id) {
    perror("Could not find the interface");
    return -1;
  }

#if 0
  struct bpf_object *object = bpf_object__open("block.o");
  int loaded = bpf_object__load(object);

  if (loaded < 0) {
    perror("Failed to load the object (a)");
    return 0;
  }

  printf("Here.\n");


	struct xdp_program *program = xdp_program__from_bpf_obj(object, "xdp");
#endif

	struct xdp_program *program = xdp_program__open_file("block.o", "xdp", NULL);
  if (!program) {
    //bpf_object__close(object);
    perror("Failed to load the object (b)");
    return 0;
  }

	int result = xdp_program__attach(program, interface_id, XDP_MODE_SKB, 0);
  if (result) {
    perror("Failed to attach");
    //bpf_object__close(object);
    return 0;
	}

	struct bpf_object *bpf_object = xdp_program__bpf_obj(program);
  int map_fd = bpf_object__find_map_fd_by_name(bpf_object, "dropped_packets");

  sleep(15);

  if (map_fd) {
    uint32_t key = 0;
    uint64_t result = 0;
    if (bpf_map_lookup_elem(map_fd, &key, &result)< 0) {
      perror("Failed to look up the element");
    }
        printf("result: %lu\n", result);
  }

	result = xdp_program__detach(program, interface_id, XDP_MODE_SKB, 0);
  if (result) {
    perror("Failed to detach");
    return 0;
	}
	xdp_program__close(program);
  return 0;
}

