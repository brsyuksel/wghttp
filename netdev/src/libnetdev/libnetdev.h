#ifndef LIBNETDEV_H
#define LIBNETDEV_H

#include <arpa/inet.h>

#define IP_PREFIX_MAXLEN 4
#define IP_NETMASK_STRLEN (INET6_ADDRSTRLEN + IP_PREFIX_MAXLEN + 1)

typedef enum {
    LIBNETDEV_ERR_NOMEM = 1,
    LIBNETDEV_ERR_CTL_SOCKET_FAILED,
    LIBNETDEV_ERR_NETLINK_SOCKET_FAILED,
    LIBNETDEV_ERR_GET_DEV_FLAGS_FAILED,
    LIBNETDEV_ERR_SET_DEV_FLAGS_FAILED,
    LIBNETDEV_ERR_INVALID_IP_STR,
    LIBNETDEV_ERR_INVALID_IP,
    LIBNETDEV_ERR_INVALID_IP_PREFIX,
    LIBNETDEV_ERR_DEV_IP_SET_FAILED,
    LIBNETDEV_ERR_DEV_NETMASK_SET_FAILED,
    LIBNETDEV_ERR_DEV_NOT_FOUND,
    LIBNETDEV_ERR_NETLINK_SEND_FAILED,
    LIBNETDEV_ERR_GETIFADDRS_FAILED,
} libnetdev_error;

typedef struct libnetdev_ip {
    char ipv4_addr[IP_NETMASK_STRLEN];
    char ipv6_addr[IP_NETMASK_STRLEN];
} libnetdev_ip;

int libnetdev_get_ip(const char *device_name, libnetdev_ip **ip);

int libnetdev_set_ip(const char *device_name, libnetdev_ip *ip);

int libnetdev_up(const char *device_name);

void libnetdev_free_ip(libnetdev_ip *ip);

#endif
