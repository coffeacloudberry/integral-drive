release:
	for fc in fc32 fc33 fc34 ; do \
		docker image build -f Dockerfile_$$fc -t integral-drive-$$fc . && \
		docker rm -f integral-drive-$$fc ; \
		docker run --name integral-drive-$$fc -td integral-drive-$$fc && \
		docker cp integral-drive-$$fc:/releases . && \
		docker stop integral-drive-$$fc ; \
	done
	rpm --addsign releases/rpm/integral-drive*.rpm
	gpg -a --yes --detach-sig --default-key happy-dude@keemail.me releases/deb/integral-drive*.deb
