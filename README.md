# Arkime Docker Compose

A Docker Compose setup for [Arkime](http://arkime.com/) that supports GPU-accelerated Python Arkime parsers and easy integration of custom plugins. The stack uses [NVIDIA Container Toolkit](https://github.com/NVIDIA/nvidia-container-toolkit) to expose GPUs to Python Arkime parsers, allowing compute-intensive parsing tasks to leverage [CUDA](https://docs.nvidia.com/cuda/index.html). The compose files and Dockerfiles are structured so you can:

- enable GPU access per service with `--gpus`/runtime settings,
- build Python Arkime parser images containing required libraries,
- mount or install custom plugins without modifying Arkime core,
- and run the entire stack locally or in CI with minimal changes.

## Getting started

Use the special hostname `host.docker.internal` for ES_OS_HOST if OpenSearch/Elasticsearch is running on the same host.
You may need to specify a network mode for docker, such as `--network=host`.

Set environment variables to configure the container. (`ARKIME>__<config>=<value>` for default section or `ARKIME_<section>__<config>=<value>`)
These variables take precedence over configuration file settings.

### Dev commands

> REF: `https://docs.opensearch.org/latest/install-and-configure/install-opensearch/docker/`
> NOTE: max virtual memory areas vm.max_map_count [65530] is too low, increase to at least [262144]; for more information see [https://www.elastic.co/guide/en/elasticsearch/reference/8.17/bootstrap-checks-max-map-count.html]

Temp -> `sudo sysctl -w vm.max_map_count=262144` -> `sudo reboot`
Permanently -> `sudo vi /etc/sysctl.conf` -> add `vm.max_map_count=262144`, `net.core.rmem_max=134217728`, `net.core.wmem_max=134217728` -> `sudo sysctl -p`

```sh
git pull
git submodule update --init

sudo chown -R 1000:1000 ./db
sudo rm -rf ./db/main/os/*
sudo rm -rf ./db/node-1/os/*
# (Optional) Fresh start
sudo rm -rf ./etc/.initialized

source .env

docker compose --progress=plain -f docker-compose.cuda.yml build --no-cache=true`, `docker compose --progress=plain -f docker-compose.cuda.yml --profile optional build kime-docs --no-cache=true
docker compose -f docker-compose.cuda.yml up -d
docker compose -f docker-compose.cuda.yml down
docker compose -f docker-compose.cuda.yml restart arkime-capture
```

### Documentation

```sh
sudo apt update
sudo apt install -y ruby-full build-essential zlib1g-dev

echo '# install ruby gems to ~/gems' >> ~/.bashrc
echo 'export GEM_HOME="$HOME/gems"' >> ~/.bashrc
echo 'export PATH="$HOME/gems/bin:$PATH"' >> ~/.bashrc
source ~/.bashrc

# gem install bundler jekyll --user-install
gem install bundler jekyll

# test
jekyll new mysite
cd mysite
bundle exec jekyll serve

bundle install --path vendor/bundle
bundle exec jekyll serve
```

### What kind of packet capture speeds can arkime-capture handle? 

On modern commodity hardware, achieving throughput of 5 Gbps or more is easy, depending largely on the number of CPUs allocated to Arkime and the other tasks the machine is handling. The bottleneck in performance is almost always the speed of storing PCAP to disk! If your disks or RAID can't keep up, you either need to not save as much PCAP using Arkime Rules and other options, select a faster RAID configuration and disks, or give Arkime dedicate disks. For further details, refer to the Architecture and Multiple Host sections. Arkime supports the utilization of multiple threads for both packet acquisition and packet processing.

A simple method to test a local RAID devicee:

```sh
dd bs=256k count=50000 if=/dev/zero of=/THE_ARKIME_PCAP_DIR/test oflag=direct
```

To test a NAS, leave off the `oflag=direct` and make sure you test with at least 3x the amount of memory so that cache isn't a factor:

```sh
dd bs=256k count=150000 if=/dev/zero of=/THE_ARKIME_PCAP_DIR/test
```

The output represents the maximum disk performance. If you wish to obtain a more accurate assessment, run several tests and average the results. To avoid packet loss, it's advisable to operate Arkime at no more than 80% of the maximum disk performance. For systems utilizing RAID, aiming for about 60% of this performance metric can further minimize issues, especially during RAID rebuilds. It's important to note that network throughput is typically measured in bits, whereas disk performance is gauged in bytes, requiring the conversion of these measurements for accurate comparison.
