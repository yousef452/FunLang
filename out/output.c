#include <stdio.h>
#include <gc.h>
#include <string.h>

int main() {
GC_INIT();
for (int i = 12; i < 14; i++) {
printf("%i\n", i);
}
return 0;
}
