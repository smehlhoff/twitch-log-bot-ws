#!/bin/bash

useradd -m $SERVER_USERNAME && \
    echo "$SERVER_USERNAME:$SERVER_PASSWORD" | chpasswd

usermod -aG sudo $SERVER_USERNAME

usermod -s /bin/bash $SERVER_USERNAME

/usr/sbin/sshd -D