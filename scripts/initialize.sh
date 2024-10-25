#!/bin/bash

# Add a network namespace named "test".
sudo ip netns add test

# Add a pair of virtual ethernet devices "veth0" and "ceth0" respectively.
sudo ip link add veth0 type veth peer name ceth0

# Add "ceth0" virtual network interface to "test" namespace and set everything up.
sudo ip link set ceth0 netns test
sudo ip link set veth0 up
sudo ip link set ceth0 up

# Assign IP addresses to virtual interfaces.
sudo ip addr add 192.168.0.1/24 dev veth0
sudo ip addr add 