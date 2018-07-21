#include <sys/ioctl.h>
#include <stdbool.h>

bool tiocsctty(int fd) {
    return -1 != ioctl(fd, TIOCSCTTY, 0);
}
