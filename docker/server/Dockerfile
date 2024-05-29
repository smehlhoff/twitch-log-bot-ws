FROM ubuntu:22.04

RUN apt-get update \
    && apt-get install -y \
        cron \
        curl \
        sudo \
        nano \
        git \
        openssh-server \
        iputils-ping \
        build-essential \
    && apt-get clean

RUN ssh-keygen -A \
    && mkdir -p /run/sshd

COPY ./docker/server/entrypoint.sh /entrypoint.sh
RUN chmod +x /entrypoint.sh

EXPOSE 22

ENTRYPOINT ["/entrypoint.sh"]
