#include "libwgshim.h"

#include <net/if.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#include "wireguard.h"

void libwgshim_from_wg_device(wg_device *wgdev, libwgshim_device *dev) {
    strncpy(dev->name, wgdev->name, IF_NAMESIZE);
    dev->port = wgdev->listen_port;

    uint64_t peers = 0;
    for (struct wg_peer *p = wgdev->first_peer; p != NULL; p = p->next_peer) {
        peers++;
    }
    dev->peers = peers;

    wg_key_to_base64(dev->private_key, wgdev->private_key);
    wg_key_to_base64(dev->public_key, wgdev->public_key);
}

int libwgshim_get_device(const char *device_name, libwgshim_device **dev) {
    wg_device *wgdev = NULL;
    if (wg_get_device(&wgdev, device_name) != 0) {
        return LIBWGSHIM_ERR_DEV_NOT_FOUND;
    }

    *dev = calloc(1, sizeof(libwgshim_device));
    if (!*dev) {
        wg_free_device(wgdev);
        return LIBWGSHIM_ERR_NOMEM;
    }

    libwgshim_from_wg_device(wgdev, *dev);
    wg_free_device(wgdev);
    return 0;
}

char *libwgshim_list_device_names() {
    return wg_list_device_names();
}

int libwgshim_create_device(const char *device_name, uint16_t port, libwgshim_device **dev) {
    if (wg_add_device(device_name) != 0) {
        return LIBWGSHIM_ERR_DEV_ADD_FAILED;
    }

    wg_device *wgdev = NULL;
    if (wg_get_device(&wgdev, device_name) != 0) {
        return LIBWGSHIM_ERR_DEV_NOT_FOUND;
    }

    wg_generate_private_key(wgdev->private_key);
    wg_generate_public_key(wgdev->public_key, wgdev->private_key);
    wgdev->listen_port = port;
    wgdev->flags = WGDEVICE_HAS_PRIVATE_KEY | WGDEVICE_HAS_PUBLIC_KEY | WGDEVICE_HAS_LISTEN_PORT;

    if (wg_set_device(wgdev) != 0) {
        wg_free_device(wgdev);
        return LIBWGSHIM_ERR_DEV_SET_FAILED;
    }

    *dev = calloc(1, sizeof(libwgshim_device));
    if (!*dev) {
        wg_free_device(wgdev);
        return LIBWGSHIM_ERR_NOMEM;
    }

    libwgshim_from_wg_device(wgdev, *dev);
    wg_free_device(wgdev);

    return 0;
}

int libwgshim_delete_device(const char *device_name) {
    return wg_del_device(device_name);
}

wg_allowedip *to_wg_allowedip(libwgshim_allowed_ip *allowed_ip_head) {
    wg_allowedip *head = NULL, *last = NULL;

    libwgshim_allowed_ip *current = allowed_ip_head;
    while (current) {
        char ip_str[40], cidr_str[4];
        uint8_t is_ipv6 = strchr(current->ip_addr, ':') ? true : false;

        char *slash = strchr(current->ip_addr, '/');
        size_t ip_len = slash - current->ip_addr;
        size_t cidr_len = strlen(current->ip_addr) - ip_len - 1;

        strncpy(ip_str, current->ip_addr, ip_len);
        ip_str[ip_len] = '\0';
        strncpy(cidr_str, slash + 1, cidr_len);
        cidr_str[cidr_len] = '\0';

        int cidr_val = atoi(cidr_str);
        if (cidr_val < 0 || cidr_val > (is_ipv6 ? 128 : 32)) {
            continue;
        }

        wg_allowedip *wg_ip = calloc(1, sizeof(wg_allowedip));
        if (!wg_ip) {
            continue;
        }

        int inet_res;
        wg_ip->cidr = (uint8_t)cidr_val;
        if (is_ipv6) {
            wg_ip->family = AF_INET6;
            inet_res = inet_pton(AF_INET6, ip_str, &wg_ip->ip6);
        } else {
            wg_ip->family = AF_INET;
            inet_res = inet_pton(AF_INET, ip_str, &wg_ip->ip4);
        }

        if (inet_res != 1) {
            free(wg_ip);
            continue;
        }

        if (!head) {
            head = wg_ip;
        }

        if (last) {
            last->next_allowedip = wg_ip;
        }
        last = wg_ip;

        current = current->next;
    }

    return head;
}

void libwgshim_from_wg_peer(wg_peer *wgpeer, wg_key private_key,
                            libwgshim_allowed_ip *allowed_ip_head, libwgshim_peer *peer) {
    wg_key_to_base64(peer->public_key, wgpeer->public_key);
    wg_key_to_base64(peer->private_key, private_key);
    wg_key_to_base64(peer->preshared_key, wgpeer->preshared_key);
    peer->persistent_keepalive_interval = wgpeer->persistent_keepalive_interval;
    peer->allowed_ip = allowed_ip_head;
}

int libwgshim_add_peer(const char *device_name, libwgshim_allowed_ip *allowed_ip_head,
                       uint16_t persistent_keepalive_interval, libwgshim_peer **peer) {
    wg_device *wgdev = NULL;
    if (wg_get_device(&wgdev, device_name) != 0) {
        return LIBWGSHIM_ERR_DEV_NOT_FOUND;
    }

    wg_peer *p = calloc(1, sizeof(wg_peer));
    if (!p) {
        wg_free_device(wgdev);
        return LIBWGSHIM_ERR_NOMEM;
    }

    wg_key private_key;
    wg_generate_private_key(private_key);

    p->flags = WGPEER_HAS_PUBLIC_KEY | WGPEER_HAS_PRESHARED_KEY;
    if (persistent_keepalive_interval > 0) {
        p->flags |= WGPEER_HAS_PERSISTENT_KEEPALIVE_INTERVAL;
    }
    wg_generate_public_key(p->public_key, private_key);
    wg_generate_preshared_key(p->preshared_key);
    p->persistent_keepalive_interval = persistent_keepalive_interval;

    // assigning peer first can help to free both in any err case.
    if (!wgdev->first_peer) {
        wgdev->first_peer = p;
    } else {
        wg_peer *last_peer = wgdev->last_peer;
        last_peer->next_peer = p;
        wgdev->last_peer = p;
    }

    wg_allowedip *wg_ip = to_wg_allowedip(allowed_ip_head);
    if (!wg_ip) {
        wg_free_device(wgdev);
        return LIBWGSHIM_ERR_NOMEM;
    }
    p->first_allowedip = wg_ip;
    wg_allowedip *current = wg_ip;
    while (current) {
        p->last_allowedip = current;
        current = current->next_allowedip;
    }

    if (wg_set_device(wgdev) != 0) {
        wg_free_device(wgdev);
        return LIBWGSHIM_ERR_DEV_SET_FAILED;
    }

    *peer = calloc(1, sizeof(libwgshim_peer));
    if (!*peer) {
        wg_free_device(wgdev);
        return LIBWGSHIM_ERR_NOMEM;
    }

    libwgshim_from_wg_peer(p, private_key, allowed_ip_head, *peer);
    wg_free_device(wgdev);
    return 0;
}

void wg_endpoint_str(wg_endpoint *endpoint, char *buf, size_t size) {
    char ip_str[INET6_ADDRSTRLEN];

    if (!endpoint) {
        return;
    }

    if (endpoint->addr.sa_family == AF_INET) {
        inet_ntop(AF_INET, &endpoint->addr4.sin_addr, ip_str, ENDPOINT_STRLEN);
        uint16_t port = ntohs(endpoint->addr4.sin_port);
        snprintf(buf, size, "%s:%d", ip_str, port);

    } else if (endpoint->addr.sa_family == AF_INET6) {
        inet_ntop(AF_INET6, &endpoint->addr6.sin6_addr, ip_str, ENDPOINT_STRLEN);
        uint16_t port = ntohs(endpoint->addr6.sin6_port);
        snprintf(buf, size, "[%s]:%d", ip_str, port);
    }
}

void libwgshim_from_wg_peer_list(wg_peer *wgpeer, libwgshim_peer *peer) {
    wg_key_to_base64(peer->public_key, wgpeer->public_key);
    wg_key_to_base64(peer->preshared_key, wgpeer->preshared_key);
    peer->last_handshake_time = wgpeer->last_handshake_time.tv_sec;
    peer->persistent_keepalive_interval = wgpeer->persistent_keepalive_interval;
    peer->rx = wgpeer->rx_bytes;
    peer->tx = wgpeer->tx_bytes;

    char endpoint_str[ENDPOINT_STRLEN];
    memset(endpoint_str, 0, ENDPOINT_STRLEN);

    wg_endpoint_str(&wgpeer->endpoint, endpoint_str, ENDPOINT_STRLEN);
    if (endpoint_str) {
        strcpy(peer->endpoint, endpoint_str);
    }

    libwgshim_allowed_ip *allowed_ip_head = NULL, *allowed_ip_last = NULL;
    wg_allowedip *ip = wgpeer->first_allowedip;
    while (ip) {
        char ip_str[ALLOWED_IP_STRLEN];

        if (ip->family == AF_INET) {
            inet_ntop(AF_INET, &ip->ip4, ip_str, ALLOWED_IP_STRLEN);
        } else if (ip->family == AF_INET6) {
            inet_ntop(AF_INET6, &ip->ip6, ip_str, ALLOWED_IP_STRLEN);
        }

        uint8_t cidr = ip->cidr;

        libwgshim_allowed_ip *aip = calloc(1, sizeof(libwgshim_allowed_ip));
        snprintf(aip->ip_addr, ALLOWED_IP_STRLEN, "%s/%hhu", ip_str, cidr);

        if (!allowed_ip_head) {
            allowed_ip_head = aip;
        }

        if (allowed_ip_last) {
            allowed_ip_last->next = aip;
        }
        allowed_ip_last = aip;

        ip = ip->next_allowedip;
    }

    peer->allowed_ip = allowed_ip_head;
}

int libwgshim_list_peers(const char *device_name, libwgshim_peer **peer_head) {
    wg_device *wgdev = NULL;
    if (wg_get_device(&wgdev, device_name) != 0) {
        return LIBWGSHIM_ERR_DEV_NOT_FOUND;
    }

    *peer_head = calloc(1, sizeof(libwgshim_peer));
    if (!*peer_head) {
        wg_free_device(wgdev);
        return LIBWGSHIM_ERR_NOMEM;
    }

    wg_peer *p = wgdev->first_peer;
    libwgshim_peer *prev = NULL, *current = NULL;
    while (p) {
        if (!current) {
            current = prev = *peer_head;
        } else {
            current = calloc(1, sizeof(libwgshim_peer));
            prev->next = current;
            prev = current;
        }

        libwgshim_from_wg_peer_list(p, current);

        p = p->next_peer;
    }

    wg_free_device(wgdev);
    return 0;
}

int libwgshim_delete_peer(const char *device_name, const char *public_key) {
    wg_device *wgdev = NULL;
    if (wg_get_device(&wgdev, device_name) != 0) {
        return LIBWGSHIM_ERR_DEV_NOT_FOUND;
    }

    uint8_t found = 0;
    wg_peer *peer = wgdev->first_peer;
    while (peer) {
        wg_key_b64_string peer_pk;
        wg_key_to_base64(peer_pk, peer->public_key);

        if (strcmp(public_key, peer_pk) == 0) {
            peer->flags |= WGPEER_REMOVE_ME;
            found = 1;
            break;
        }

        peer = peer->next_peer;
    }

    if (!found) {
        wg_free_device(wgdev);
        return LIBWGSHIM_ERR_PEER_NOT_FOUND;
    }

    if (wg_set_device(wgdev) != 0) {
        wg_free_device(wgdev);
        return LIBWGSHIM_ERR_DEV_SET_FAILED;
    }

    wg_free_device(wgdev);
    return 0;
}

void libwgshim_free_device(libwgshim_device *dev) {
    if (!dev) {
        return;
    }
    free(dev);
}

void libwgshim_free_peer(libwgshim_peer *peer) {
    while (peer) {
        libwgshim_peer *next_peer = peer->next;

        libwgshim_allowed_ip *ip = peer->allowed_ip;
        while (ip) {
            libwgshim_allowed_ip *next_ip = ip->next;
            free(ip);
            ip = next_ip;
        }

        free(peer);
        peer = next_peer;
    }
}
