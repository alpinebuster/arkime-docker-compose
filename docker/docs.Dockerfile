### STAGE-1
ARG RUBY_VERSION=3.3
FROM ruby:$RUBY_VERSION-slim AS builder
# FROM ruby:$RUBY_VERSION-slim-bullseye

RUN \
  --mount=type=cache,target=/var/cache/apt,sharing=locked \
  --mount=type=cache,target=/var/lib/apt,sharing=locked \
  apt-get update -qq && apt-get install -yqq \
  build-essential \
  curl \
  locales && \
  rm -rf /var/lib/apt/lists/*
RUN mkdir -p /usr/src/app && chmod -R 777 /usr/src/app
WORKDIR /usr/src/app
# NOTE: The current docker compose context is `project_root_dir/docs/`
COPY . .

# Ref: https://docs.docker.com/build/cache/optimize/#use-cache-mounts
# RUN --mount=type=cache,target=/root/.gem,sharing=locked \
#   # bundle config set force_ruby_platform true && \
#   # sed -i 's|source "https://rubygems.org"|source "https://gems.ruby-china.com/"|g' "./Gemfile" && \
#   # sed -i 's|source "https://rubygems.org"|source "https://mirrors.aliyun.com/rubygems/"|g' "./Gemfile" && \
#   bundle install --gemfile=Gemfile --retry 12
RUN \
  --mount=type=cache,target=/root/.gem,sharing=locked \
  sed -i 's|gem "github-pages", group: :jekyll_plugins|#gem "github-pages", group: :jekyll_plugins|g' "./Gemfile" && \
  echo "gem 'bigdecimal'" >> ./Gemfile && \
  echo "gem 'csv'" >> ./Gemfile && \
  echo "gem 'zeitwerk'" >> ./Gemfile && \
  echo "gem 'webrick'" >> ./Gemfile && \
  echo "gem 'rexml'" >> ./Gemfile && \
  echo "gem 'nokogiri'" >> ./Gemfile && \
  echo "gem 'base64'" >> ./Gemfile && \
  echo "gem 'logger'" >> ./Gemfile && \
  echo "gem 'pathutil'" >> ./Gemfile && \
  echo "gem 'jekyll-include-cache', '= 0.2.1'" >> ./Gemfile && \
  echo "gem 'jekyll-octicons', '~> 14.2'" >> ./Gemfile && \
  NOKOGIRI_USE_SYSTEM_LIBRARIES=true bundle install && \
  # 🤷 https://stackoverflow.com/questions/66113639/jekyll-serve-throws-no-implicit-conversion-of-hash-into-integer-error
  sed -i.bak 's/, kwd/, **kwd/' $(gem which pathutil)
RUN \
  echo "en_US UTF-8" > /etc/locale.gen && \
  locale-gen en-US.UTF-8

ENV LANG en_US.UTF-8
ENV LANGUAGE en_US.UTF-8
ENV LC_ALL en_US.UTF-8

RUN curl -o ./assets/bootstrap.bundle.min.js https://stackpath.bootstrapcdn.com/bootstrap/4.1.3/js/bootstrap.bundle.min.js && \
    curl -o ./assets/jquery.min.js https://ajax.googleapis.com/ajax/libs/jquery/2.2.4/jquery.min.js && \
    sed -i 's|https://ajax.googleapis.com/ajax/libs/jquery/2.2.4/jquery.min.js|assets/jquery.min.js|' ./_includes/navbar.html && \
    sed -i 's|https://stackpath.bootstrapcdn.com/bootstrap/4.1.3/js/bootstrap.bundle.min.js|assets/bootstrap.bundle.min.js|' ./_includes/navbar.html && \
    sed -i '/<script async src="https:\/\/www.googletagmanager.com\/gtag\/js?id=UA-137788272-1"><\/script>/d' ./arkimeet.html && \
    sed -i '/<script async src="https:\/\/www.googletagmanager.com\/gtag\/js?id=UA-137788272-1"><\/script>/d' ./arkimeetus.html && \
    sed -i '/<script async src="https:\/\/www.googletagmanager.com\/gtag\/js?id=UA-137788272-1"><\/script>/d' ./cont3xt.html && \
    sed -i '/<script async src="https:\/\/www.googletagmanager.com\/gtag\/js?id=UA-137788272-1"><\/script>/d' ./demo.html && \
    sed -i '/<script async src="https:\/\/www.googletagmanager.com\/gtag\/js?id=UA-137788272-1"><\/script>/d' ./downloads-old.html && \
    sed -i '/<script async src="https:\/\/www.googletagmanager.com\/gtag\/js?id=UA-137788272-1"><\/script>/d' ./downloads.html && \
    sed -i '/<script async src="https:\/\/www.googletagmanager.com\/gtag\/js?id=UA-137788272-1"><\/script>/d' ./estimators.html && \
    sed -i '/<script async src="https:\/\/www.googletagmanager.com\/gtag\/js?id=UA-137788272-1"><\/script>/d' ./faq.html && \
    sed -i '/<script async src="https:\/\/www.googletagmanager.com\/gtag\/js?id=UA-137788272-1"><\/script>/d' ./index.html && \
    sed -i '/<script async src="https:\/\/www.googletagmanager.com\/gtag\/js?id=UA-137788272-1"><\/script>/d' ./mini-arkimeetus.html && \
    sed -i '/<script async src="https:\/\/www.googletagmanager.com\/gtag\/js?id=UA-137788272-1"><\/script>/d' ./parliament.html && \
    sed -i '/<script async src="https:\/\/www.googletagmanager.com\/gtag\/js?id=UA-137788272-1"><\/script>/d' ./release-v5.html && \
    sed -i '/<script async src="https:\/\/www.googletagmanager.com\/gtag\/js?id=UA-137788272-1"><\/script>/d' ./settings.html && \
    sed -i '/<script async src="https:\/\/www.googletagmanager.com\/gtag\/js?id=UA-137788272-1"><\/script>/d' ./v3release.html && \
    sed -i '/<script async src="https:\/\/www.googletagmanager.com\/gtag\/js?id=UA-137788272-1"><\/script>/d' ./v4release.html && \
    sed -i '/<script async src="https:\/\/www.googletagmanager.com\/gtag\/js?id=UA-137788272-1"><\/script>/d' ./_includes/head.html
  RUN JEKYLL_ENV=production bundle exec jekyll build
# EXPOSE 80
# CMD ["bundle", "exec", "jekyll", "serve", "-H", "0.0.0.0", "-P", "80"]


### STAGE-2
FROM nginx:latest

# Copy the generated static files from our Ruby container and placing them in the default nginx directory
COPY --from=builder /usr/src/app/_site /usr/share/nginx/html/_site

# Instructing docker that we wish to expose port 80
EXPOSE 80

# Specifying the command that will be run when the container starts, this case running nginx in the foreground.
CMD ["nginx", "-g", "daemon off;"]
