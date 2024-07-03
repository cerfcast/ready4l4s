#include <linux/bpf.h>
#include <linux/if_ether.h>
#include <linux/ip.h>
#include <bpf/bpf_helpers.h>
#include <arpa/inet.h>

#define MAC_HEADER_SIZE 14
#define member_address(source_struct, source_member)                           \
  ((void *)(((char *)source_struct) +                                          \
            offsetof(typeof(*source_struct), source_member)))

#define member_read(destination, source_struct, source_member)                 \
  do {                                                                         \
    bpf_probe_read(destination, sizeof(source_struct->source_member),          \
                   member_address(source_struct, source_member));              \
  } while (0)

struct {
  __uint(type, BPF_MAP_TYPE_ARRAY);
  __type(key, __u32);
  __type(value, long unsigned int);
  __uint(max_entries, 2);
} dropped_packets SEC(".maps");

SEC("xdp")
int selective_drop(struct xdp_md *md) {

  void *data_end = (void*)(long)md->data_end;
  void *data = (void*)(long)md->data;
  struct ethhdr *eth =data;
	__u16 eth_proto = 0;

    

  __u32 hkey = 0;
  long unsigned int zero = 0;
  long unsigned int *pm = bpf_map_lookup_elem(&dropped_packets, &hkey);

  if (!pm) {
    bpf_map_update_elem(&dropped_packets, &hkey, &zero, BPF_NOEXIST);
    pm = bpf_map_lookup_elem(&dropped_packets, &hkey);
    if (!pm)
      return -1;
  }

	if (data + sizeof(struct ethhdr) > data_end) {
    return XDP_PASS;
  }

  eth_proto = eth->h_proto;

	if (eth_proto == htons(ETH_P_IP)) {
    struct iphdr *iphdr = (struct iphdr *)(data + sizeof(struct ethhdr));

		        if (iphdr + 1 > (struct iphdr *)data_end) {
            return XDP_PASS;
        }

		__u8 tos = iphdr->tos;
		//__u8 tos = 1;
    //if (tos) {
      *pm +=1;
		//}
  }

  return XDP_PASS;
}

char LICENSE[] SEC("license") = "GPL";
