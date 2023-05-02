doc:
	mkdir -p man1
	pandoc --standalone --to man integral-drive.1.md -o man1/integral-drive.1
	gzip -f man1/integral-drive.1

release: doc
	for fc in fc32 fc33 fc34 fc35 fc36 fc37 fc38 ; do \
		docker image build -f Dockerfile_$$fc -t integral-drive-$$fc . && \
		docker rm -f integral-drive-$$fc ; \
		docker run --name integral-drive-$$fc -td integral-drive-$$fc && \
		docker cp integral-drive-$$fc:/releases . && \
		docker stop integral-drive-$$fc ; \
	done
	rpm --addsign releases/rpm/integral-drive*.rpm
	gpg -a --yes --detach-sig --default-key happy-dude@keemail.me releases/deb/integral-drive*.deb
