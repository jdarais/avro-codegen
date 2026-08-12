# To build, the avro c++ library and boost must be installed.
# Alternatively, use the `podman-build` target to build inside a
# container that has all the required dependencies installed.

BUILD_IMAGE_NAME ?= avro-codegen-cpp-builder
AVRO_CODEGEN_OUTPUT_FILES = $(shell find ../../output/cpp/include -type f)

a.out: test.cpp $(AVRO_CODEGEN_OUTPUT_FILES)
	g++  -I ../../output/cpp/include test.cpp -l avrocpp

podman-image:
	podman build -t $(BUILD_IMAGE_NAME) -f Containerfile .

podman-build: podman-image
	podman run --rm \
		-v $(CURDIR)/../..:/build:Z \
		-w /build/test/cpp \
		$(BUILD_IMAGE_NAME) \
		make

clean:
	-rm a.out

