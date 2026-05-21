CC=arm-none-eabi-gcc

CFLAGS=-mcpu=cortex-m4 \
        -mthumb \
        -g \
        -O0 \
        -ffreestanding \
        -nostdlib

all:
	$(CC) $(CFLAGS) main.c -o target.elf

clean:
	rm -f target.elf