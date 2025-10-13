# Arkime Docker Cluster

## Getting started
Use the special hostname `host.docker.internal` for ES_OS_HOST if OpenSearch/Elasticsearch is running on the same host.
You may need to specify a network mode for docker, such as `--network=host`.

Set environment variables to configure the container. (`ARKIME>__<config>=<value>` for default section or `ARKIME_<section>__<config>=<value>`)
These variables take precedence over configuration file settings.
