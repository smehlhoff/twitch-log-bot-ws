#!/bin/bash

useradd -m $SERVER_USERNAME && \
    echo "$SERVER_USERNAME:$SERVER_PASSWORD" | chpasswd

usermod -aG sudo $SERVER_USERNAME
usermod -s /bin/bash $SERVER_USERNAME

sed -i "s/#Port 22/Port 2222/" /etc/ssh/sshd_config

/usr/sbin/sshd -D
