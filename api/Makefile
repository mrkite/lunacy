all: iigs.dat mac.dat dos.dat

iigs.dat: docmaker/docmaker iigs
	$(MAKE) -C docmaker
	./docmaker/docmaker iigs

mac.dat: docmaker/docmaker mac
	$(MAKE) -C docmaker
	./docmaker/docmaker mac

dos.dat: docmaker/docmaker dos
	$(MAKE) -C docmaker
	./docmaker/docmaker dos

docmaker/docmaker:
	$(MAKE) -C docmaker

clean:
	rm iigs.dat mac.dat
	$(MAKE) -C docmaker clean
