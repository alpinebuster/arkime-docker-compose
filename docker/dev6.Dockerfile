ARG DOCKER_UBUNTU_VERSION=24.04
FROM ubuntu:${DOCKER_UBUNTU_VERSION}

ARG ARKIME_BRANCH=dev6
ENV ARKIME_BRANCH $ARKIME_BRANCH

LABEL org.opencontainers.image.authors="alpinebuster <imzqqq@hotmail.com>"
LABEL org.opencontainers.image.source='https://github.com/alpinebuster/arkime-docker-compose'
LABEL org.opencontainers.image.licenses='AGPL-3.0-or-later'

ENV DEBIAN_FRONTEND=noninteractive
# Ref: https://docs.docker.com/build/cache/optimize/#use-cache-mounts
RUN \
  --mount=type=cache,target=/var/cache/apt,sharing=locked \
  --mount=type=cache,target=/var/lib/apt,sharing=locked \
  sed -i 's/archive.ubuntu.com/mirrors.aliyun.com/g' /etc/apt/sources.list && \
  sed -i 's/security.ubuntu.com/mirrors.aliyun.com/g' /etc/apt/sources.list && \
  apt-get update && \
  apt-get install -y --no-install-recommends ca-certificates openssl && \
  release=$(grep VERSION_CODENAME /etc/os-release | cut -d= -f2) && \
  echo "deb https://mirrors.tuna.tsinghua.edu.cn/ubuntu ${release} main restricted universe multiverse" >> /etc/apt/sources.list && \
  echo "deb https://mirrors.tuna.tsinghua.edu.cn/ubuntu ${release}-updates main restricted universe multiverse" >> /etc/apt/sources.list && \
  echo "deb https://mirrors.tuna.tsinghua.edu.cn/ubuntu ${release}-backports main restricted universe multiverse" >> /etc/apt/sources.list && \
  echo "deb https://mirrors.tuna.tsinghua.edu.cn/ubuntu ${release}-security main restricted universe multiverse" >> /etc/apt/sources.list && \
  apt-get update && \
  apt-get install -y lsb-release build-essential make python3-pip git libtest-differences-perl sudo wget apt-utils tzdata libnl-genl-3-dev zstd


# Declare default `ARG`s
# Initialize is used to reset the environment from scratch and rebuild a new ES Stack
ARG INITIALIZE_DB=true
# Wipe is the same as initialize except it keeps users intact
ARG WIPE_DB=false
ARG ES_OS_HOST=es_os-main
ARG ES_OS_PORT=9200
ARG ES_OS_USERNAME=elastic
ARG ES_OS_PASSWORD=elastic_pwd
ARG ARKIME_HOSTNAME=localhost
ARG ARKIME_INTERFACE=enp6s0
ARG ARKIME_USERNAME=arkime
ARG ARKIME_PASSWORD=arkime_pwd
ARG ARKIME_INSTALL_DIR="/opt/arkime"
ARG ARKIME_APP_DIR="/opt/arkime/app"
ARG CAPTURE=on
ARG VIEWER=on
ARG PARLIAMENT=on
ARG CONT3XT=off
ARG WISE=off

# Declare `ENV` vars for each `ARG`
ENV INITIALIZE_DB $INITIALIZE_DB
ENV WIPE_DB $WIPE_DB
ENV ES_OS_HOST $ES_OS_HOST
ENV ES_OS_PORT $ES_OS_PORT
ENV ES_OS_USERNAME $ES_OS_USERNAME
ENV ES_OS_PASSWORD $ES_OS_PASSWORD
ENV ARKIME_HOSTNAME $ARKIME_HOSTNAME
ENV ARKIME_INTERFACE $ARKIME_INTERFACE
ENV ARKIME_USERNAME $ARKIME_USERNAME
ENV ARKIME_PASSWORD $ARKIME_PASSWORD
ENV ARKIME_INSTALL_DIR $ARKIME_INSTALL_DIR
ENV ARKIME_APP_DIR $ARKIME_APP_DIR
ENV CAPTURE $CAPTURE
ENV VIEWER $VIEWER
ENV PARLIAMENT $PARLIAMENT
ENV CONT3XT $CONT3XT
ENV WISE $WISE


RUN \
  --mount=type=cache,target=/var/cache/apt,sharing=locked \
  --mount=type=cache,target=/var/lib/apt,sharing=locked \
  git clone https://github.com/arkime/arkime.git && \
  (cd arkime; git checkout $ARKIME_BRANCH; ./easybutton-build.sh --nothirdparty --kafka --rminstall)
RUN \
    ldd capture/capture && \
    export PATH=${ARKIME_INSTALL_DIR}/bin:$PATH && \
    (cd /arkime; INSTALL_BUNDLE=bundle make install) && \
    # NOTE: create the etc/oui.txt, this is slow
    # It's needed for importing PCAPs. This step is omitted during 'Configure', because ARKIME_INET=no is set in 'startarkime.sh'
    "${ARKIME_INSTALL_DIR}/bin/arkime_update_geo.sh"

# Add scripts
# NOTE: The current docker compose context is `project_root_dir/`
COPY ./etc ${ARKIME_INSTALL_DIR}/etc/
COPY ./scripts ${ARKIME_APP_DIR}/
RUN chmod 755 ${ARKIME_APP_DIR}/*.sh
ENV PATH="${ARKIME_INSTALL_DIR}/bin:${ARKIME_APP_DIR}:${PATH}"


# ja4plus
# PWD: `/arkime/` -> `/arkime/`
# Switching to the parent directory (cd ..) in a subshell only affects that subshell and does not impact the main shell's state. This means that after this step, the working directory of the main shell remains unchanged.
# 
# Running Multiple Commands
# Using a semicolon (;) allows multiple commands to be executed in sequence within the parentheses. All commands are executed in the same subshell, maintaining the dependencies between the commands.
WORKDIR /arkime

RUN (cd .. ; git clone https://github.com/alpinebuster/arkime-ja4.git ./ja4)
RUN cp ../ja4/ja4plus.c capture/plugins && \
    # `/arkime/capture/plugins/` 
    (cd capture/plugins; make) && \
    (cd ../ja4; make) && \
    # `/arkime/tests/`
    (cd tests; ./tests.pl --extra "-o plugins=ja4plus.so" ../../ja4/pcap/*.pcap)
RUN mv capture/plugins/ja4plus*.so ${ARKIME_INSTALL_DIR}/plugins/ && \
    rm -f capture/plugins/ja4plus.c


RUN useradd -u 2000 opensearch; \
(cd / ; \
 wget https://artifacts.opensearch.org/releases/core/opensearch/2.7.0/opensearch-min-2.7.0-linux-x64.tar.gz; \
 tar xf opensearch-min-2.7.0-linux-x64.tar.gz; \
 chown -R opensearch opensearch-2.7.0; \
 rm -f opensearch-min-2.7.0-linux-x64.tar.gz \
);


WORKDIR ${ARKIME_INSTALL_DIR}
# Clean up deps to reduce image size
RUN \
  rm -rf /arkime && \
  # wget https://apt.llvm.org/llvm.sh && \
  # bash llvm.sh 18 && \
  # apt-get install -y ruby-dev && \
  # gem install --no-document fpm && \
  rm -rf /var/lib/apt/lists/*

EXPOSE 8005
EXPOSE 3220

ENTRYPOINT ["/opt/arkime/app/start_arkime.sh"]
