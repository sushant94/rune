global _main

section .text
	main:
		; Some instructions, not really relevant for our case.
	
	write:
		; Funtion epilogue and prologue skipped. Only function body.
		; Unrelated asm is skipped for clarity of this example.
		; Assume buf is assigned to be at rbp - 0xa.
		; in x86_64, address of buf will be in rdi when this function is
		; called.
		lea rax, [rbp - 0xa]
		add rax, rdi
		mov rax, rsi
		ret


