/**
 * @file libnetdev.h
 * @brief Public API for interacting with network devices and their IP configurations.
 * 
 * This library provides functions to get and set IP addresses, netmasks, and device states
 * for network interfaces. It abstracts the underlying system calls and provides a simple
 * interface for managing network devices programmatically.
 */

#ifndef LIBNETDEV_H
#define LIBNETDEV_H

#include <arpa/inet.h>

// Maximum length for ip prefix addition in CIDR notation. Since Ipv6 can have a prefix length of up to 3 digits,
// we define a maximum length of 4 to accommodate the prefix and the null terminator.
#define IP_PREFIX_MAXLEN 4

// Maximum length for an IPv4 or IPv6 address string in CIDR notation
#define IP_NETMASK_STRLEN (INET6_ADDRSTRLEN + IP_PREFIX_MAXLEN + 1)

/**
 * @brief Error codes returned by libnetdev functions.
 */
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

/**
 * @brief Represents an IP configuration for a network device.
 * 
 * Contains both IPv4 and IPv6 addresses in CIDR notation.
 */
typedef struct libnetdev_ip {
    char ipv4_addr[IP_NETMASK_STRLEN];
    char ipv6_addr[IP_NETMASK_STRLEN];
} libnetdev_ip;

/**
 * @brief Retrieves the IP configuration for a given network device.
 * 
 * @param device_name Name of the network device (e.g., "eth0", "wg0")
 * @param ip Output pointer to the libnetdev_ip structure that will be filled with the device's IP
 *           configuration.
 * @return 0 on success, non-zero on failure.
 */
int libnetdev_get_ip(const char *device_name, libnetdev_ip **ip);

/**
 * @brief Sets the IP configuration for a given network device.
 * 
 * This function sets both the IPv4 and IPv6 addresses in CIDR notation. If an address is not set,
 * it should be an empty string.
 * 
 * @param device_name Name of the network device (e.g., "eth0", "wg0")
 * @param ip Pointer to the libnetdev_ip structure containing the new IP configuration.
 * @return 0 on success, non-zero on failure.
 */
int libnetdev_set_ip(const char *device_name, libnetdev_ip *ip);

/**
 * @brief Retrieves the netmask for a given network device.
 * 
 * @param device_name Name of the network device (e.g., "eth0", "wg0")
 * @return 0 on success, non-zero on failure.
 */
int libnetdev_up(const char *device_name);

/**
 * @brief Frees the memory allocated for a libnetdev_ip structure.
 * 
 * @param ip Pointer to the libnetdev_ip structure to be freed
 */
void libnetdev_free_ip(libnetdev_ip *ip);

#endif
