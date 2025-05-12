#include <stdlib.h>
#include <stdio.h>
#include <string.h>
#include <stdbool.h>
#include <unistd.h>
#include <sys/socket.h>
#include <sys/ioctl.h>
#include <net/if.h>
#include <arpa/inet.h>
#include <ifaddrs.h>
#include <linux/netlink.h>
#include <linux/rtnetlink.h>

#include "libnetdev.h"

int count_prefix_bits_v4(uint32_t netmask) {
    int bits = 0;
    netmask = ntohl(netmask);
    while (netmask & 0x80000000) {
        bits++;
        netmask <<= 1;
    }
    return bits;
}

int count_prefix_bits_v6(struct in6_addr *mask) {
    int bits = 0;
    for (int i = 0; i < 16; i++) {
        uint8_t byte = mask->s6_addr[i];
        for (int j = 7; j >= 0; j--) {
            if (byte & (1 << j)) {
                bits++;
            } else {
                return bits;
            }
        }
    }
    return bits;
}

int libnetdev_get_ip(const char *device_name, libnetdev_ip **ip) {
    struct ifaddrs *ifaddr = NULL, *ifa = NULL;

    if(getifaddrs(&ifaddr) == -1) {
        return LIBNETDEV_ERR_GETIFADDRS_FAILED;
    }

    *ip = calloc(1, sizeof(libnetdev_ip));
    if(!ip) {
        freeifaddrs(ifaddr);
        return LIBNETDEV_ERR_NOMEM;
    }

    for(ifa = ifaddr; ifa != NULL; ifa = ifa->ifa_next) {
        if(!ifa->ifa_addr || strcmp(ifa->ifa_name, device_name) != 0) {
            continue;
        }

        int family = ifa->ifa_addr->sa_family;
        char ip_str[IP_NETMASK_STRLEN] = {0};

        if(family == AF_INET) {
            struct sockaddr_in *addr = (struct sockaddr_in *)ifa->ifa_addr;
            struct sockaddr_in *netmask = (struct sockaddr_in *)ifa->ifa_netmask;

            inet_ntop(AF_INET, &addr->sin_addr, ip_str, sizeof(ip_str));
            int prefix = count_prefix_bits_v4(netmask->sin_addr.s_addr);
            snprintf((*ip)->ipv4_addr, sizeof((*ip)->ipv4_addr), "%s/%d", ip_str, prefix);
        }

        if(family == AF_INET6) {
            struct sockaddr_in6 *addr = (struct sockaddr_in6 *)ifa->ifa_addr;
            struct sockaddr_in6 *netmask = (struct sockaddr_in6 *)ifa->ifa_netmask;

            inet_ntop(AF_INET6, &addr->sin6_addr, ip_str, sizeof(ip_str));
            int prefix = count_prefix_bits_v6(&netmask->sin6_addr);
            snprintf((*ip)->ipv6_addr, sizeof((*ip)->ipv6_addr), "%s/%d", ip_str, prefix);
        }
    }

    freeifaddrs(ifaddr);
    return 0;
}

int split_ip_and_prefix(const char *ip_prefix_str, char *ip_buf, size_t ip_buf_size, char *prefix_buf, size_t prefix_buf_size) {
    bool is_ipv6 = strchr(ip_prefix_str, ':') != NULL;
    const char *slash = strchr(ip_prefix_str, '/');

    if(slash) {
        size_t ip_len = slash - ip_prefix_str;
        if(ip_len >= ip_buf_size) {
            return LIBNETDEV_ERR_INVALID_IP_STR;
        }

        strncpy(ip_buf, ip_prefix_str, ip_len);
        ip_buf[ip_len] = '\0';
        
        size_t prefix_len = strlen(ip_prefix_str) - ip_len - 1;
        if(prefix_len >= prefix_buf_size) {
            return LIBNETDEV_ERR_INVALID_IP_STR;
        }

        strncpy(prefix_buf, slash + 1, prefix_len);
        prefix_buf[prefix_len] = '\0';
    } else {
        if(strlen(ip_prefix_str) >= ip_buf_size) {
            return LIBNETDEV_ERR_INVALID_IP_STR;
        }

        strncpy(ip_buf, ip_prefix_str, ip_buf_size - 1);
        ip_buf[ip_buf_size - 1] = '\0';

        strncpy(prefix_buf, is_ipv6 ? "128" : "32", prefix_buf_size - 1);
        prefix_buf[prefix_buf_size - 1] = '\0';
    }

    return 0;
}

int set_ipv6(const char *device_name, const char *ipv6_str, const char *prefix_str) {
    struct in6_addr ipv6 = {0};
    if(inet_pton(AF_INET6, ipv6_str, &ipv6) != 1) {
        return LIBNETDEV_ERR_INVALID_IP;
    }

    int prefix = atoi(prefix_str);
    if(prefix < 0 || prefix > 128) {
        return LIBNETDEV_ERR_INVALID_IP_PREFIX;
    }

    int if_index = if_nametoindex(device_name);
    if(if_index == 0) {
        return LIBNETDEV_ERR_DEV_NOT_FOUND;
    }

    int fd = socket(AF_NETLINK, SOCK_RAW, NETLINK_ROUTE);
    if(fd < 0) {
        return LIBNETDEV_ERR_NETLINK_SOCKET_FAILED;
    }

    char buf[512];
    memset(buf, 0, sizeof(buf));

    struct nlmsghdr *nlh = (struct nlmsghdr *) buf;
    struct ifaddrmsg *ifa = (struct ifaddrmsg *)(nlh + 1);

    nlh->nlmsg_len = NLMSG_LENGTH(sizeof(*ifa));
    nlh->nlmsg_type = RTM_NEWADDR;
    nlh->nlmsg_flags = NLM_F_REQUEST | NLM_F_CREATE | NLM_F_REPLACE;
    nlh->nlmsg_seq = 1;
    nlh->nlmsg_pid = getpid();

    ifa->ifa_family = AF_INET6;
    ifa->ifa_prefixlen = prefix;
    ifa->ifa_index = if_index;
    ifa->ifa_scope = 0;
    ifa->ifa_flags = IFA_F_PERMANENT;

    struct rtattr *rta = (struct rtattr *)(((char *) nlh) + NLMSG_ALIGN(nlh->nlmsg_len));
    rta->rta_type = IFA_ADDRESS;
    rta->rta_len = RTA_LENGTH(sizeof(struct in6_addr));
    memcpy(RTA_DATA(rta), &ipv6, sizeof(struct in6_addr));
    nlh->nlmsg_len = NLMSG_ALIGN(nlh->nlmsg_len) + RTA_LENGTH(sizeof(struct in6_addr));

    struct rtattr *rta_local = (struct rtattr *)(((char *) nlh) + NLMSG_ALIGN(nlh->nlmsg_len));
    rta_local->rta_type = IFA_LOCAL;
    rta_local->rta_len = RTA_LENGTH(sizeof(struct in6_addr));
    memcpy(RTA_DATA(rta_local), &ipv6, sizeof(struct in6_addr));
    nlh->nlmsg_len = NLMSG_ALIGN(nlh->nlmsg_len) + RTA_LENGTH(sizeof(struct in6_addr));

    struct sockaddr_nl addr = {
        .nl_family = AF_NETLINK
    };

    struct iovec iov = {
        .iov_base = nlh,
        .iov_len = nlh->nlmsg_len
    };

    struct msghdr msg = {
        .msg_name = &addr,
        .msg_namelen = sizeof(addr),
        .msg_iov = &iov,
        .msg_iovlen = 1
    };

    if(sendmsg(fd, &msg, 0) < 0) {
        close(fd);
        return LIBNETDEV_ERR_NETLINK_SEND_FAILED;
    }

    close(fd);
    return 0;
}

int set_ipv4(const char *device_name, const char *ipv4_str, const char *prefix_str) {
    struct sockaddr_in addr = {0};
    struct sockaddr_in netmask = {0};

    addr.sin_family = AF_INET;
    netmask.sin_family = AF_INET;

    if(inet_pton(AF_INET, ipv4_str, &addr.sin_addr) != 1) {
        return LIBNETDEV_ERR_INVALID_IP;
    }

    int prefix = atoi(prefix_str);
    if(prefix < 0 || prefix > 32) {
        return LIBNETDEV_ERR_INVALID_IP_PREFIX;
    }

    uint32_t mask = prefix == 0 ? 0 : htonl(0xFFFFFF << (32 - prefix));
    netmask.sin_addr.s_addr = mask;

    int fd = socket(AF_INET, SOCK_DGRAM, 0);
    if(fd < 0) {
        return LIBNETDEV_ERR_CTL_SOCKET_FAILED;
    }

    struct ifreq ifr = {0};
    strncpy(ifr.ifr_name, device_name, IF_NAMESIZE - 1);
    memcpy(&ifr.ifr_addr, &addr, sizeof(struct sockaddr_in));

    if(ioctl(fd, SIOCSIFADDR, &ifr) < 0) {
        close(fd);
        return LIBNETDEV_ERR_DEV_IP_SET_FAILED;
    }

    memcpy(&ifr.ifr_netmask, &netmask, sizeof(struct sockaddr_in));
    if(ioctl(fd, SIOCSIFNETMASK, &ifr) < 0) {
        close(fd);
        return LIBNETDEV_ERR_DEV_NETMASK_SET_FAILED;
    }

    close(fd);
    return 0;
}

int libnetdev_set_ip(const char *device_name, libnetdev_ip *ip) {
    char ip_buf[IP_NETMASK_STRLEN];
    char prefix_buf[IP_PREFIX_MAXLEN];

    if(ip->ipv4_addr[0]) {
        int res = split_ip_and_prefix(ip->ipv4_addr, ip_buf, IP_NETMASK_STRLEN, prefix_buf, IP_PREFIX_MAXLEN);
        if(res != 0) {
            return res;
        }

        res = set_ipv4(device_name, ip_buf, prefix_buf);
        if(res != 0) {
            return res;
        }
    }

    if(ip->ipv6_addr[0]) {
        int res = split_ip_and_prefix(ip->ipv6_addr, ip_buf, IP_NETMASK_STRLEN, prefix_buf, IP_PREFIX_MAXLEN);
        if(res != 0) {
            return res;
        }

        res = set_ipv6(device_name, ip_buf, prefix_buf);
        if(res != 0) {
            return res;
        }
    }
    return 0;
}

int libnetdev_up(const char *device_name) {
    int fd = socket(AF_INET, SOCK_DGRAM, 0);
    if(fd < 0) {
        return LIBNETDEV_ERR_CTL_SOCKET_FAILED;
    }

    struct ifreq ifr = {0};

    strncpy(ifr.ifr_name, device_name, IF_NAMESIZE - 1);

    if(ioctl(fd, SIOCGIFFLAGS, &ifr) < 0) {
        close(fd);
        return LIBNETDEV_ERR_GET_DEV_FLAGS_FAILED;
    }

    ifr.ifr_flags |= IFF_UP;

    if(ioctl(fd, SIOCSIFFLAGS, &ifr) < 0) {
        close(fd);
        return LIBNETDEV_ERR_SET_DEV_FLAGS_FAILED;
    }

    close(fd);
    return 0;
}

void libnetdev_free_ip(libnetdev_ip *ip) {
    if(!ip) {
        return;
    }
    free(ip);
}
