# syntax=docker/dockerfile:1

# We inherit from an ubuntu image that is modified to support long-running services
# consisting of multiple processes. To learn how to define a background services, 
# consult https://github.com/phusion/baseimage-docker
FROM quay.io/pypa/manylinux_2_24_x86_64:latest

RUN apt-get update
RUN apt-get install --yes --no-install-recommends clang z3
