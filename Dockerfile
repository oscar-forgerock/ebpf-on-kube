FROM rust:1 as ebpf-on-kube-build

RUN apt-get update && apt-get install -y \
 clang \
 gcc \
 libbpf-dev \
 llvm \
 bpftool

WORKDIR /Code/ebpf-on-kube
COPY ./ /Code/ebpf-on-kube

RUN --mount=type=cache,target=/usr/local/cargo/registry \
    C_INCLUDE_PATH=/usr/include/aarch64-linux-gnu/ cargo build

ENTRYPOINT ["/Code/ebpf-on-kube/target/debug/ebpf-on-kube"]
