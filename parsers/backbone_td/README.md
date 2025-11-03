# A lightweight method for detecting extremely low-proportion dt in backbone networks

Detecting Tor Traffic using lightweight operator fusion-oriented CNN.

## Prerequisites

Remove Outdated Signing Key:

```sh
sudo apt-key del 7fa2af80
sudo apt install software-properties-common
sudo add-apt-repository ppa:deadsnakes/ppa
```

### python
Automatic installer:
```sh
sudo apt install zlib1g zlib1g-dev libssl-dev libbz2-dev liblzma-dev libsqlite3-dev libreadline-dev python3-dev python3-venv
curl https://pyenv.run | bash

pip config set global.index-url https://pypi.tuna.tsinghua.edu.cn/simple
pip install setuptools wheel poetry
poetry lock
poetry install
poetry install --extras gpu --extras local
poetry install -vvvv
poetry shell
```

### rust
```sh
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

### Graphviz
Generating dot and SVG graphs requires Graphviz, an open source graph visualization software:
`sudo apt --yes install graphviz`

### libpcap & tshark
Install them to fix the error "= note: /usr/bin/ld: cannot find -lpcap: No such file or directory":
`sudo apt-get install -y libpcap-dev tshark`

### cuda-driver
1. To install the open kernel module flavor:
`sudo apt-get install -y nvidia-open`

2. To install the legacy kernel module flavor:
`sudo apt-get install -y cuda-drivers`

### cuda-toolkit
1. Use deb(network)
Install the new cuda-keyring package:
```sh
wget https://developer.download.nvidia.com/compute/cuda/repos/ubuntu2404/x86_64/cuda-keyring_1.1-1_all.deb
sudo dpkg -i cuda-keyring_1.1-1_all.deb
sudo apt-get update
sudo apt-get -y install cuda-toolkit-12-6
```

2. Use dev(local):
```sh
wget https://developer.download.nvidia.com/compute/cuda/repos/ubuntu2404/x86_64/cuda-ubuntu2404.pin
sudo mv cuda-ubuntu2404.pin /etc/apt/preferences.d/cuda-repository-pin-600
wget https://developer.download.nvidia.com/compute/cuda/12.6.3/local_installers/cuda-repo-ubuntu2404-12-6-local_12.6.3-560.35.05-1_amd64.deb
sudo dpkg -i cuda-repo-ubuntu2404-12-6-local_12.6.3-560.35.05-1_amd64.deb
sudo cp /var/cuda-repo-ubuntu2404-12-6-local/cuda-*-keyring.gpg /usr/share/keyrings/
sudo apt-get update
sudo apt-get -y install cuda-toolkit-12-6
```

Environment setup:
`export PATH=/usr/local/cuda-12.6/bin${PATH:+:${PATH}}`

### tensorrt
[tensorrt](https://developer.nvidia.com/tensorrt/download/10x)
[DEB](https://developer.nvidia.com/downloads/compute/machine-learning/tensorrt/10.7.0/local_repo/nv-tensorrt-local-repo-ubuntu2204-10.7.0-cuda-12.6_1.0-1_amd64.deb)

```sh
wget https://developer.nvidia.com/downloads/compute/machine-learning/tensorrt/10.7.0/local_repo/nv-tensorrt-local-repo-ubuntu2404-10.7.0-cuda-12.6_1.0-1_amd64.deb
sudo dpkg -i nv-tensorrt-local-repo-ubuntu2404-10.7.0-cuda-12.6_1.0-1_amd64.deb
sudo cp /var/nv-tensorrt-local-repo-ubuntu2404-10.7.0-cuda-12.6_1.0-1_amd64/*-keyring.gpg /usr/share/keyrings/
sudo apt-get update
```

For the full C++ and Python runtimes:
`sudo apt-get install tensorrt`
For only running TensorRT C++ applications:
`sudo apt-get install tensorrt-libs`
For also building TensorRT C++ applications:
`sudo apt-get install tensorrt-dev`

## Getting Started

Run the code below to generate low-proportion Tor dataset:
- `sudo su` then `nohup bash ./gen_lowptor.sh >gen_lowptor-dataset_name-tor_ratio.log 2>&1 &`
- `./kill_all_gen.sh`

To execute feature extraction / model training:
- `poetry install` or `poetry install -vvv`

Rust:
- In the root directory of the workspace, use the `cargo build` command to build the entire workspace.
- Build a specific package: `cargo build --package package_a`
- Build in release mode: `cargo build --release --no-default-features`
- Clean build artifacts: `cargo clean`
- Run a package in the workspace: `cargo run --package package_a`
- `cargo test --no-default-features`
- `nohup cargo run <tor_src_dataset> <tor_ratio> >collaborative_filtering-<tor_src_dataset>-<tor_ratio>.log 2>&1 &`

mdbook:
Build the book and start a local webserver:
```sh
mdbook serve --open
```

Python:
- `nohup python3 main.py >/dev/null 2>&1 &`
- `ps -def | grep  main.py | grep -v grep | awk '{print $2}' | xargs kill`

### tensorboard

If you want to view the tensorboard visualization interface locally, you can start the local server using the following command:

```bash
tensorboard --logdir=runs
```

## References

### Models
- `MobileNet` uses depthwise separable convolutions to reduce the number of parameters and computational cost while maintaining accuracy.
- `EfficientNet` is a family of models that scale up the network width, depth, and resolution in a balanced way. It uses a compound scaling method to optimize performance and efficiency.
- `GhostNet` introduces a novel Ghost module that generates more feature maps from cheap operations, allowing the network to maintain performance while reducing computational cost.
- `ShuffleNet` is another lightweight architecture that uses pointwise group convolutions and channel shuffling to improve the efficiency of the network.

### tcpreplay
Common options:
  - --intf1=<interface>: Specify the network interface to send traffic.
  - --pps=<packets_per_second>: Specify the number of packets to send per second.
  - --mbps=<megabits_per_second>: Specify the bandwidth to use.
  - --loop=<count>: Specify the number of times to replay the traffic.
  - --quiet: Run in quiet mode, suppressing detailed output.
  - --verbose: Output detailed debugging information.

```sh
# Create a veth pair
sudo ip link add veth_tcpreplay type veth peer name veth_tcpdump

# Start the two interfaces
sudo ip link set veth_tcpreplay up
sudo ip link set veth_tcpdump up

sudo tcpreplay -i veth_tcpreplay /path/to/src_traffic.pcap
sudo tcpdump -i veth_tcpdump -w /path/to/output.pcap
sudo tcpdump -i veth_tcpdump -tttt
sudo tcpdump -i veth_tcpdump -v

# Replay traffic at a rate of 1000 packets per second
sudo tcpreplay -i veth_tcpreplay --pps=1000 /path/to/src_traffic.pcap
# Replay traffic at a bandwidth of 10 Mbps
sudo tcpreplay -i veth_tcpreplay --mbps=10 /path/to/src_traffic.pcap
```
