TARGET=$(shell rustc -vV | awk '$$1 == "host:"{print $$2}')
BUILD_DIR=${CURDIR}/build
VERSION=$(shell cat Cargo.toml | awk 'BEGIN{FS="[ \"]"}$$1 == "application_version"{print $$4;exit}')
RELEASE_FILENAME_POSTFIX=

all: release

release: _make-build-dir
	cargo build --release --target ${TARGET}
	@ cp ./target/${TARGET}/release/sssh ${BUILD_DIR}/sssh-${VERSION}-${TARGET}${RELEASE_FILENAME_POSTFIX}
	@ ls -sh ${BUILD_DIR}/sssh-*

deb: _make-build-dir
	cargo deb --target ${TARGET}
	@ cp ./target/${TARGET}/debian/*.deb ${BUILD_DIR}/sssh-${VERSION}-${TARGET}${RELEASE_FILENAME_POSTFIX}.deb

dev: _make-build-dir
	cargo build --target ${TARGET}
	@ cp ./target/${TARGET}/debug/sssh ${BUILD_DIR}/sssh-${VERSION}-${TARGET}-dev${RELEASE_FILENAME_POSTFIX}
	@ ls -sh ${BUILD_DIR}/sssh-*dev*

lint:
	cargo fmt --verbose --check
	cargo check --target ${TARGET}
	#cargo clippy --no-deps --target ${TARGET}

test:
	cargo test --target ${TARGET}


_make-build-dir:
	@ mkdir -p ${BUILD_DIR}
