# ebpf-on-kube

A set of eBPF probes that people can work on

# pre-requisites
lima
cargo
bpfman.io


# Build docker image
1. create ubuntu vm `limactl start --name ubuntu lima-ubuntu.yml`(note: modify `lima-ubuntu.yml` mount-point to checkout path)
2. run vm shell `limactl shell ubuntu` project is mount
3. build image `docker build --tag ebpf-on-kube:latest . 
4. push to registry

# Deploy
TODO (follow https://bpfman.io/main/getting-started/example-bpf-k8s/) 