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
COPY ./docker/server/setup.sh /home/setup.sh

RUN chmod +x /entrypoint.sh
RUN chmod +x /home/setup.sh

EXPOSE 22

ENTRYPOINT ["/entrypoint.sh"]
