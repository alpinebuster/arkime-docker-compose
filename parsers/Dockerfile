# syntax=docker/dockerfile:1-labs

# 
# NOTE:
# 
# At project's root dir:
#   1. Make sure that the `zzz/arkime:$ARKIME_VERSION` image is already available locally
#      `DOCKER_BUILDKIT=1 docker build --progress=plain -t zzz/arkime:latest --build-arg DOCKER_UBUNTU_VERSION=24.04 --build-arg PYTHON=python3.12 -f docker/dev6.Dockerfile .`
#   2. Make sure that the nvidia driver and `nvidia-container-toolkit` are installed on the host machine, see `parsers/README.md` for how to install it
#      `DOCKER_BUILDKIT=1 docker build --progress=plain -t zzz/arkime-cuda:latest --build-arg DOCKER_UBUNTU_VERSION=24.04 --build-arg FROM=zzz/arkime:latest -f parsers/Dockerfile .`
#   3. Run `nvidia-smi` to test
#      `docker run --network arkime_es_os -e INITIALIZE_DB=false -e WIPE_DB=false -e CAPTURE=off -e VIEWER=off -e ARKIME__elasticsearch=http://${ES_OS_USERNAME}:${ES_OS_PASSWORD}@${ES_OS_HOST}:${ES_OS_PORT} -v $(pwd)/scripts/start_arkime.sh:/opt/arkime/app/start_arkime.sh -it --rm --runtime=nvidia --gpus all zzz/arkime-cuda:latest`
#      `docker run -it --rm --entrypoint /bin/bash --gpus all zzz/arkime-cuda:latest`
#      `docker run -it --rm --entrypoint /bin/bash --gpus all zzz/arkime-cuda:latest -c nvidia-smi`
#      `docker run --rm --entrypoint /bin/bash --gpus all zzz/arkime-cuda:latest -c "command -v nvidia-smi || echo 'nvidia-smi not found'; ls -l /usr/bin | grep nvidia || true" /usr/bin/nvidia-smi`
#      `docker run --rm --gpus all nvidia/cuda:13.0.1-cudnn-runtime-ubuntu${DOCKER_UBUNTU_VERSION} bash -c "command -v nvidia-smi || echo 'nvidia-smi not found'; ls -l /usr/bin | grep nvidia || true"`
# 

ARG ARKIME_VERSION=latest
ARG FROM=zzz/arkime:$ARKIME_VERSION
ARG DOCKER_UBUNTU_VERSION=24.04
ARG PYTHON_VERSION=3.12

# first of all, we create a base image with dependencies which we can copy into the
# target image. For repeated rebuilds, this is much faster than apt installing
# each time.
# `nvidia/cuda:13.0.1-cudnn-devel-ubuntu${DOCKER_UBUNTU_VERSION}`
FROM nvidia/cuda:13.0.1-cudnn-runtime-ubuntu${DOCKER_UBUNTU_VERSION} AS deps_base

# now build the final image, based on the the regular *arkime* docker image
FROM $FROM
  LABEL org.opencontainers.image.authors="alpinebuster <imzqqq@hotmail.com>"
  LABEL org.opencontainers.image.source='https://github.com/alpinebuster/arkime-docker-compose'
  LABEL org.opencontainers.image.description="DTA with CUDA & Arkime 6 dev builds for Python parsers"
  LABEL org.opencontainers.image.licenses='AGPL-3.0-or-later'

  ENV DEBIAN_FRONTEND=noninteractive

  # Copy the CUDA runtime, toolchain, and library files
  COPY --from=deps_base /usr/local/cuda-13.0/ /usr/local/cuda-13.0/
  COPY --from=deps_base /usr/lib/x86_64-linux-gnu /usr/lib/x86_64-linux-gnu
  COPY --from=deps_base /usr/include /usr/include
  # Copy and run commands (such as nvidia-smi, etc.)
  COPY --from=deps_base /usr/bin/nvidia* /usr/bin/

  # Environmental variables
  RUN ln -s /usr/local/cuda-13.0 /usr/local/cuda || true
  ENV PATH=/usr/local/cuda/bin:${PATH}
  ENV LD_LIBRARY_PATH=/usr/local/cuda/lib64:${LD_LIBRARY_PATH}

  RUN python3 -m venv /opt/venv
  ENV PATH="/opt/venv/bin:$PATH"

  COPY parsers/requirements.txt ${ARKIME_INSTALL_DIR}/parsers/requirements.txt
  RUN --mount=type=cache,target=/root/.cache/pip \
    python3 -m pip install --upgrade pip setuptools wheel && \
    python3 -m pip install --prefix=/install --no-deps -r ${ARKIME_INSTALL_DIR}/parsers/requirements.txt
  ENV PATH=/install/bin:${PATH}
  # NOTE: `ARG PYTHON=python3.12` is set by default in `zzz/arkime:latest` base image
  ENV PYTHONPATH=/install/lib/${PYTHON}/site-packages:${PYTHONPATH}

  # Expose viewer listener port
  EXPOSE 8005/tcp

  ENTRYPOINT ["/opt/arkime/app/start_arkime.sh"]

  # Replace the healthcheck with one which checks *all* the workers.
  # HEALTHCHECK --start-period=5s --interval=15s --timeout=5s \
  #   CMD ["/healthcheck.sh"]
