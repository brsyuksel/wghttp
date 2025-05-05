/**
 * @file libwgshim.h
 * @brief Public API for interacting with WireGuard interfaces and peers via a simplified shim layer.
 *
 * This library provides C-level structures and functions to manage WireGuard devices programmatically.
 * It includes functionality to create/delete devices, add/list/remove peers, and access configuration data.
 */

#include <arpa/inet.h>
#include <net/if.h>

#ifndef LIBWGSHIM_H
#define LIBWGSHIM_H

// Max length for a CIDR notation IPv6 address string, e.g., "ffff:...:ffff/128" + null terminator
#define ALLOWED_IP_STRLEN (INET6_ADDRSTRLEN + 5) // '/' + 3-digit CIDR + NULL

// Max length for endpoint string: "[IPv6]:65535" + null terminator
#define ENDPOINT_STRLEN (INET6_ADDRSTRLEN + 9)   // '[' + ']' + ':' + 5-digit port + NULL

// Length of base64-encoded WireGuard keys, including null terminator
#define LIBWGSHIM_B64_KEY_SIZE 45

/**
 * @brief Error codes returned by libwgshim functions.
 */
typedef enum {
    LIBWGSHIM_ERR_NOMEM = 1,
    LIBWGSHIM_ERR_DEV_NOT_FOUND,
    LIBWGSHIM_ERR_DEV_ADD_FAILED,
    LIBWGSHIM_ERR_DEV_SET_FAILED,
    LIBWGSHIM_ERR_PEER_NOT_FOUND,
} libwgshim_error;

/**
 * @brief Represents a WireGuard device (interface).
 */
typedef struct libwgshim_device {
    char name[IF_NAMESIZE];                      // Interface name (e.g., "wg0")

    uint16_t port;                               // Listening port
    uint64_t peers;                              // Number of associated peers

    char public_key[LIBWGSHIM_B64_KEY_SIZE];     // Base64-encoded public key
    char private_key[LIBWGSHIM_B64_KEY_SIZE];    // Base64-encoded private key
} libwgshim_device;

/**
 * @brief Represents an allowed IP for a peer.
 *
 * Stored as a singly linked list.
 */
typedef struct libwgshim_allowed_ip {
    char ip_addr[ALLOWED_IP_STRLEN];             // IP address in CIDR format (e.g., "10.0.0.1/32")

    struct libwgshim_allowed_ip *next;           // Pointer to next allowed IP
} libwgshim_allowed_ip;

/**
 * @brief Represents a WireGuard peer configuration.
 *
 * Peers may be organized in a linked list for batch operations.
 */
typedef struct libwgshim_peer {
    struct libwgshim_allowed_ip *allowed_ip;     // Linked list of allowed IPs for this peer

    char endpoint[ENDPOINT_STRLEN];              // Remote endpoint (e.g., "[2001:db8::1]:17079")

    int64_t last_handshake_time;                 // Timestamp of last handshake (UNIX epoch)
    uint16_t persistent_keepalive_interval;      // Keepalive interval in seconds
    uint64_t rx, tx;                             // Data counters: received and transmitted bytes

    char public_key[LIBWGSHIM_B64_KEY_SIZE];     // Base64-encoded public key
    char private_key[LIBWGSHIM_B64_KEY_SIZE];    // Base64-encoded private key
    char preshared_key[LIBWGSHIM_B64_KEY_SIZE];  // Base64-encoded preshared key

    struct libwgshim_peer *next;                 // Pointer to next peer
} libwgshim_peer;

/**
 * @brief Retrieves a WireGuard device by name.
 *
 * @param device_name Name of the device (e.g., "wg0")
 * @param dev Output pointer to the device struct
 * @return 0 on success, non-zero on failure
 */
int libwgshim_get_device(const char *device_name, libwgshim_device **dev);

/**
 * @brief Lists all WireGuard device names on the system.
 *
 * @return Null-terminated string of null-terminator-separated device names (e.g., "first\0second\0last\0\0")
 */
char *libwgshim_list_device_names();

/**
 * @brief Creates a new WireGuard device.
 *
 * @param device_name Name of the new device
 * @param port Listening port to assign
 * @param dev Output pointer to the newly created device struct
 * @return 0 on success, non-zero on failure
 */
int libwgshim_create_device(const char *device_name, uint16_t port, libwgshim_device **dev);

/**
 * @brief Deletes a WireGuard device by name.
 *
 * @param device_name Name of the device to delete
 * @return 0 on success, non-zero on failure
 */
int libwgshim_delete_device(const char *device_name);

/**
 * @brief Adds a peer to the given WireGuard device.
 *
 * @param device_name Name of the target device
 * @param allowed_ip_head Head of the allowed IP linked list
 * @param persistent_keepalive_interval Interval in seconds, or 0 to disable
 * @param peer Output pointer to the created peer
 * @return 0 on success, non-zero on failure
 */
int libwgshim_add_peer(const char *device_name, libwgshim_allowed_ip *allowed_ip_head, uint16_t persistent_keepalive_interval, libwgshim_peer **peer);

/**
 * @brief Lists all peers associated with a given WireGuard device.
 *
 * @param device_name Name of the device
 * @param peer_head Output pointer to the head of the peer linked list
 * @return 0 on success, non-zero on failure
 */
int libwgshim_list_peers(const char *device_name, libwgshim_peer **peer_head);

/**
 * @brief Deletes a peer from a WireGuard device using its public key.
 *
 * @param device_name Name of the device
 * @param public_key Base64-encoded public key of the peer to remove
 * @return 0 on success, non-zero on failure
 */
int libwgshim_delete_peer(const char *device_name, const char *public_key);

/**
 * @brief Frees memory allocated for a libwgshim_device struct.
 *
 * Use this to release the memory returned by libwgshim_get_device() or libwgshim_create_device().
 *
 * @param dev Pointer to the device struct to be freed. May be NULL.
 */
void libwgshim_free_device(libwgshim_device *dev);

/**
 * @brief Frees memory allocated for a WireGuard peer and its associated allowed IPs.
 *
 * Use this function to free a single peer struct obtained from libwgshim_add_peer()
 * or from a list returned by libwgshim_list_peers(). This also recursively frees any
 * linked allowed IP entries associated with the peer.
 *
 * @param peer Pointer to the peer to be freed. May be NULL.
 */
void libwgshim_free_peer(libwgshim_peer *peer);

#endif // LIBWGSHIM_H
