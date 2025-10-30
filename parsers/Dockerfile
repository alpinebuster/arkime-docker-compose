# syntax=docker/dockerfile:1-labs

# NOTE:
# 
# At project root dir:
#   1. Make sure that the `zzz/arkime:$ARKIME_VERSION` image is already available locally
#      `DOCKER_BUILDKIT=1 docker build --progress=plain -t zzz/arkime:latest -f docker/dev6.Dockerfile .`
#   2. Make sure that the nvidia driver and `nvidia-container-toolkit` are installed on the host machine, see `parsers/README.md` for how to install it
#      `DOCKER_BUILDKIT=1 docker build --progress=plain -t zzz/arkime-cuda:latest --build-arg FROM=zzz/arkime:latest -f parsers/Dockerfile .`
#   3. Run nvidia-smi to test
#      `docker run --rm --runtime=nvidia --gpus all zzz/arkime-cuda:latest nvidia-smi`
# 

ARG ARKIME_VERSION=latest
ARG FROM=zzz/arkime:$ARKIME_VERSION
ARG DEBIAN_VERSION=trixie
ARG PYTHON_VERSION=3.13

# first of all, we create a base image with dependencies which we can copy into the
# target image. For repeated rebuilds, this is much faster than apt installing
# each time.
FROM nvidia/cuda:12.6.3-cudnn-devel-ubuntu24.04 AS deps_base

# now build the final image, based on the the regular *arkime* docker image
FROM $FROM
  LABEL org.opencontainers.image.authors="alpinebuster <imzqqq@hotmail.com>"
  LABEL org.opencontainers.image.source='https://github.com/alpinebuster/arkime-docker-compose'
  LABEL org.opencontainers.image.description="DTA with Arkime 6 dev builds for Python parsers"
  LABEL org.opencontainers.image.licenses='AGPL-3.0-or-later'

  ENV DEBIAN_FRONTEND=noninteractive

  # Copy the CUDA runtime, toolchain, and library files
  COPY --from=deps_base /usr/local/cuda-12.6/ /usr/local/cuda-12.6/
  COPY --from=deps_base /usr/lib/x86_64-linux-gnu /usr/lib/x86_64-linux-gnu
  COPY --from=deps_base /usr/include /usr/include
  # Copy and run commands (such as nvidia-smi, etc.)
  COPY --from=deps_base /usr/bin/nvidia* /usr/bin/

  # Environmental variables
  RUN ln -s /usr/local/cuda-12.6 /usr/local/cuda || true
  ENV PATH=/usr/local/cuda/bin:${PATH}
  ENV LD_LIBRARY_PATH=/usr/local/cuda/lib64:${LD_LIBRARY_PATH}

  RUN \
    --mount=type=cache,target=/var/cache/apt,sharing=locked \
    --mount=type=cache,target=/var/lib/apt,sharing=locked \
    apt-get update && apt-get install -y --no-install-recommends \
      python3-distutils \
      curl \
    && curl -sS https://bootstrap.pypa.io/get-pip.py | python3 \
    && rm -rf /var/lib/apt/lists/*

  COPY parsers/requirements.txt ${ARKIME_INSTALL_DIR}/parsers/requirements.txt
  RUN --mount=type=cache,target=/root/.cache/pip \
    python3 -m pip install --upgrade pip setuptools wheel && \
    python3 -m pip install --prefix=/install --no-deps -r ${ARKIME_INSTALL_DIR}/parsers/requirements.txt
  ENV PATH=/install/bin:${PATH}
  # NOTE: `ARG PYTHON=python3.13` is set by `zzz/arkime:latest` base image
  ENV PYTHONPATH=/install/lib/${PYTHON}/site-packages:${PYTHONPATH}

  # Expose viewer listener port
  EXPOSE 8005/tcp

  ENTRYPOINT ["/opt/arkime/app/start_arkime.sh"]

  # Replace the healthcheck with one which checks *all* the workers.
  # HEALTHCHECK --start-period=5s --interval=15s --timeout=5s \
  #   CMD ["/healthcheck.sh"]
