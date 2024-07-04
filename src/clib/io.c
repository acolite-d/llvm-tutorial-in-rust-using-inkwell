#ifdef _WIN32
#define DLLEXPORT __declspec(dllexport)
#else
#define DLLEXPORT
#endif

#include <stdio.h>

// putchard - putchar that takes a double as ascii code, prints it with a newline and returns 0.
extern DLLEXPORT double putchard(double X) {
    fputc((char)X, stderr);
    fputc(10, stderr);
    return 0;
}

// printd, prints the double value to stdout
extern DLLEXPORT double printd(double d) {
    printf("\"%f\"\n", d);
    return 0;
}