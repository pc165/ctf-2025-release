FROM debian:latest

RUN apt update -y && \
apt install socat -y && \
apt clean -y

ADD bashrc /home/ctf/.bashrc

ADD flag.txt /home/ctf/

RUN chown -R root:root /home/ctf

ENTRYPOINT ["socat", "TCP-LISTEN:4445,reuseaddr,fork", "EXEC:'/bin/bash -li',pty,stderr,setsid,sane"]
