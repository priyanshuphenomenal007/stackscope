TARGET=target

CC=arm-none-eabi-gcc

CFLAGS= \
    -mcpu=cortex-m4 \
    -mthumb \
    -g \
    -O0 \
    -ffreestanding \
    -fno-inline \
    -fno-omit-frame-pointer \
    -nostdlib

all:
	$(CC) $(CFLAGS) -Wl,-e,main -o $(TARGET).elf main.c

clean:
	rm -f *.elf