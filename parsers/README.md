# Python Arkime Module

The Python Arkime module has high level methods to register callbacks for packet processing.
Use the setting disablePython=true to disable Python support in Arkime.

## Constants
VERSION : String - The Arkime version as a string
CONFIG_PREFIX : String - The Arkime install prefix, usually /opt/arkime
API_VERSION : Integer - The Arkime API version from arkime.h

## Getting Started
Create a `example.py` file at `/opt/arkime/parsers/` folder.

### nvidia-docker

- `sudo fuser -v /dev/nvidia*`
- `sudo nvidia-docker run -p 11050:11250 --name=pytorch_container_name -v ~/workspace:/root/workspace -v /data:/root/data --shm-size 64g --device /dev/nvidiactl --device /dev/nvidia-uvm --device /dev/nvidia0 --device /dev/nvidia1 --device /dev/nvidia2 --device /dev/nvidia3 -it pytorch_image_name /bin/bash`

**Installing on Ubuntu and Debian**

The following steps can be used to setup NVIDIA Container Toolkit on Ubuntu LTS (18.04, 20.04, and 22.04) and Debian (Stretch, Buster) distributions.

**Setting up NVIDIA Container Toolkit**

Set up the package repository and the GPG key:

```sh
curl -fsSL https://nvidia.github.io/libnvidia-container/gpgkey | sudo gpg --dearmor -o /usr/share/keyrings/nvidia-container-toolkit-keyring.gpg \
  && curl -s -L https://nvidia.github.io/libnvidia-container/stable/deb/nvidia-container-toolkit.list | \
    sed 's#deb https://#deb [signed-by=/usr/share/keyrings/nvidia-container-toolkit-keyring.gpg] https://#g' | \
    sudo tee /etc/apt/sources.list.d/nvidia-container-toolkit.list
```

> Note that in some cases the downloaded list file may contain URLs that do not seem to match the expected value of distribution which is expected as packages may be used for all compatible distributions. As examples:

For distribution values of ubuntu20.04 or ubuntu22.04 the file will contain ubuntu18.04 URLs
For a distribution value of debian11 the file will contain debian10 URLs

> Note: if running apt update after configuring repositories raises an error regarding a conflict in the Signed-By option, see the relevant troubleshooting section.

Install the `nvidia-container-toolkit` package (and dependencies) after updating the package listing:

```sh
sudo apt-get update
sudo apt-get install -y nvidia-container-toolkit
```

Configure the Docker daemon to recognize the NVIDIA Container Runtime:

```sh
sudo nvidia-ctk runtime configure --runtime=docker
```

Restart the Docker daemon to complete the installation after setting the default runtime:

```sh
sudo systemctl restart docker docker.socket
```

At this point, a working setup can be tested by running a base CUDA container:

```sh
sudo docker run --rm --runtime=nvidia --gpus all nvidia/cuda:12.6.3-cudnn-devel-ubuntu24.04 nvidia-smi
```

If there is no `nvidia` runtime, add this Daemon configuration file to `/etc/docker/daemon.json`:

```sh
# kill docker
sudo systemctl stop docker.socket

sudo tee /etc/docker/daemon.json <<EOF
{
    "runtimes": {
        "nvidia": {
            "path": "/usr/bin/nvidia-container-runtime",
            "runtimeArgs": []
        }
    }
}
EOF

sudo systemctl restart docker
```

Run `sudo docker info|grep -i runtime` to check if the runtime is successfully added.
This should result in a console output.

#### Reinstall nvidia driver

```sh
cat /proc/driver/nvidia/version
dpkg -l | grep -i nvidia

sudo apt remove --purge "nvidia-*" -y
sudo apt-get --purge remove "*nvidia*"
sudo apt autoremove
# NOTE: must reboot to clean kernal info (to check, run `cat /proc/driver/nvidia/version`)
sudo reboot

add-apt-repository ppa:graphics-drivers/ppa
ubuntu-drivers devices
sudo ubuntu-drivers autoinstall


# Or (Recommended)
wget https://developer.download.nvidia.com/compute/cuda/repos/ubuntu$(lsb_release -rs | tr -d .)/x86_64/cuda-keyring_1.1-1_all.deb
sudo dpkg -i cuda-keyring_1.1-1_all.deb
sudo apt-get update
sudo apt-get -y install cuda


watch -n 10 nvidia-smi
```
