#include <stdio.h>
#include <gc.h>
#include <string.h>

int main() {
GC_INIT();
char* hi = (char*) GC_MALLOC(strlen("hi") + 1);
strcpy(hi, "hi");
printf("%s\n", hi);
return 0;
}
