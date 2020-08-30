#include <err.h>
#include <errno.h>
#include <fcntl.h>
#include <stdlib.h>
#include <string.h>
#include <systemd/sd-daemon.h>
#include <unistd.h>

int find_desired_fd(char** names, char* name) {
  for (int i = 0; names[i]; ++i) {
    if (strcmp(names[i], name) == 0) {
      return i;
    }
  }
  return -1;
}

int main(int argc, char** argv) {
  char **desired_fds = argv+1;
  char **execline = 0;
  for (int i=1; i<argc; ++i) {
    if (strcmp(argv[i], "--") == 0) {
      argv[i] = 0;
      execline = argv+i+1;
      break;
    }
  }
  if (execline == 0) {
    errx(1, "no --");
  }

  char **names = 0;
  int num_fds = sd_listen_fds_with_names(0, &names);
  if (num_fds < 0) {
    errx(1, "sd_listen_fds_with_names: %s", strerror(num_fds));
  }
  int *fd_map = malloc(sizeof(int) * num_fds);
  for (int i=0; i<num_fds; i++) {
    fd_map[i] = SD_LISTEN_FDS_START+i;
  }
  for(int i=0; i<num_fds; i++) {
    int fd = fd_map[i];
    int desired_fd = find_desired_fd(desired_fds, names[i]);
    if (desired_fd == -1) {
      if (close(fd) == -1) {
        err(1, "close");
      }
      warnx("no desired fd found for socket %s, ignoring", names[i]);
      continue;
    }
    if (fd == desired_fd) {
      int flags = fcntl(fd, F_GETFD);
      if (flags == -1) {
        err(1, "fcntl");
      }
      if (fcntl(fd, F_SETFD, flags & ~FD_CLOEXEC) == -1) {
        err(1, "fcntl");
      }
      warnx("%s found at %i", names[i], fd);
      continue;
    }
    if (fcntl(desired_fd, F_GETFD) != -1 || errno != EBADF) {
      if (desired_fd >= 3) {
        int moved_fd = dup(desired_fd);
        if (moved_fd == -1) {
          err(1, "dup");
        }
        fd_map[desired_fd] = moved_fd;
      }
      if (close(desired_fd) == -1) {
        err(1, "close");
      }
    }
    if (dup2(fd, desired_fd) == -1) {
      err(1, "dup2");
    }
    if (close(fd) == -1) {
      err(1, "close");
    }
    desired_fds[desired_fd] = "";
    warnx("%s duped to %i", names[i], desired_fd);
  }

  execvp(execline[0], execline);
  err(1, "execvp");
}
