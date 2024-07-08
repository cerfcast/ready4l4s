#include <arpa/inet.h>
#include <linux/bpf.h>
#include <bpf/bpf_helpers.h>
#include <linux/if_ether.h>
#include <linux/ip.h>
#include <linux/tcp.h>

#include "config.h"

struct {
  __uint(type, BPF_MAP_TYPE_ARRAY);
  __type(key, __u32);
  __type(value, long unsigned int);
  __uint(max_entries, 1);
} dropped_packets SEC(".maps");

SEC("xdp")
int selective_drop(struct xdp_md *md) {

  void *data_end = (void *)(long)md->data_end;
  void *data = (void *)(long)md->data;
  struct ethhdr *eth = data;
  __u16 eth_proto = 0;
  __u32 dropped_packets_map_key = 0;

  long unsigned int dropped_packets_map_initial_value = 0;
  long unsigned int *pm =
      bpf_map_lookup_elem(&dropped_packets, &dropped_packets_map_key);

  if (!pm) {
    bpf_map_update_elem(&dropped_packets, &dropped_packets_map_key,
                        &dropped_packets_map_initial_value, BPF_NOEXIST);
    pm = bpf_map_lookup_elem(&dropped_packets, &dropped_packets_map_key);
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

    struct tcphdr *tcphdr =
        (struct tcphdr *)(data + sizeof(struct ethhdr) + sizeof(struct iphdr));

    if (tcphdr + 1 > (struct tcphdr *)data_end) {
      return XDP_PASS;
    }

    __u8 tos = iphdr->tos & 0x3;
    if (!tos && tcphdr->dest == htons(BLOCKING_PORT)) {
#if 0
      char msg[] = "Would drop from %lu\n";
      bpf_trace_printk(msg, sizeof(msg), iphdr->saddr);
#endif
      *pm += 1;
    }

    char msg[] = "Letting packet pass from %lu\n";
    bpf_trace_printk(msg, sizeof(msg), htonl(iphdr->saddr));
    return XDP_PASS;
  }

  // This is not an ethernet packet, so YOLO.
  return XDP_PASS;
}

char LICENSE[] SEC("license") = "GPL";
