// Simple (maybe unrealistic?) example to show working of rune.
// This is not to be compiled, but rather here only to provide source level
// information for human reading.

#include <stdio.h>

void write(int where, char what)
{
	char buf[10];
	// Some computations un-related to our example.
	// Unbounded write into buffer.
	buf[where] = what;
}

int main(int argc, char **argv)
{
	int where;
	char what;

	do {
		scanf("%d %c", &where, &what);
		write(where, what);
	} while(where > 0);

	return 0;
}
