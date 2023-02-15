.Phony: all clean

all:
	sh setenv.sh
	make -C os build
	make -C os qemu

clean:
	make -C os clean
