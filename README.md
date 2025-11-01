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
