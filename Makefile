TARGET=$(shell rustc -vV | awk '$$1 == "host:"{print $$2}')
BUILD_DIR=${CURDIR}/build
VERSION=$(shell cat Cargo.toml | awk 'BEGIN{FS="[ \"]"}$$1 == "application_version"{print $$4;exit}')
CMD=${BUILD_DIR}/sssh-${VERSION}-${TARGET}${RELEASE_FILENAME_POSTFIX}
DEV_CMD=${BUILD_DIR}/sssh-${VERSION}-${TARGET}-dev${RELEASE_FILENAME_POSTFIX}
RELEASE_FILENAME_POSTFIX=



all: release


release: ${BUILD_DIR}
	cargo build --release --target ${TARGET}
	@ cp ./target/${TARGET}/release/sssh ${CMD}
	@ ls -sh ${BUILD_DIR}/sssh-*


deb: ${BUILD_DIR}
	cargo deb --target ${TARGET}
	@ cp ./target/${TARGET}/debian/*.deb ${CMD}.deb


dev: ${BUILD_DIR}
	cargo build --target ${TARGET}
	@ cp ./target/${TARGET}/debug/sssh ${DEV_CMD}
	@ ls -sh ${BUILD_DIR}/sssh-*dev*


start-dev: dev
	${DEV_CMD}


lint:
	cargo fmt --verbose --check
	cargo check --target ${TARGET}
	cargo clippy --no-deps --target ${TARGET}


test:
	cargo test --target ${TARGET}


${BUILD_DIR}:
	@ mkdir -p ${BUILD_DIR}


.PHONY: all release deb dev start-dev lint test
