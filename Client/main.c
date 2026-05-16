#include <stdio.h>
#include <string.h>

// for the cur
#include <curl/curl.h>

int main(int argc, char *argv[]) {
  // once per programm we have to init the curl
  curl_global_init(CURL_GLOBAL_DEFAULT);

  // argc is the amount of arguments
  // argv is the array of arguments
  // argv[0] is the programm name

  char *command = argv[1];

  // check if command is pull, push, etc.
  if (strcmp(command, "pl")) {
    // for pulling files, repos and also the git keys

  } else if (strcmp(command, "pshfl")) {
    // for pushing a file

  } else if (strcmp(command, "mkrp")) {
    // for making a repo
  }

  return 0;
}
