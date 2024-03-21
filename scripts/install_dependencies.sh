#!/bin/bash

# The minimum required kernel version
MIN_KERNEL_VERSION=5.11

# Function to compare kernel versions
version_ge() {
    test "$(printf '%s\n' "$@" | sort -V | head -n 1)" != "$1";
}

# Get the current kernel version
current_kernel=$(uname -r)
if version_ge $MIN_KERNEL_VERSION $current_kernel; then
    echo "Current kernel version ($current_kernel) is less than or equal to $MIN_KERNEL_VERSION. Exiting."
    exit 1
else
    echo "Current kernel version ($current_kernel) is greater than $MIN_KERNEL_VERSION. Continuing."
fi

# Function to identify the Linux distribution
identify_distro() {
    if [ -f /etc/os-release ]; then
        . /etc/os-release
        echo $ID
    else
        echo "Unknown"
    fi
}

DISTRO=$(identify_distro)

# Install `pkg-config`
case $DISTRO in
    ubuntu|debian)
        echo "Detected Ubuntu/Debian system."
        echo "Installing pkg-config..."
        sudo apt install pkg-config
        ;;
    rhel|fedora|centos)
        echo "Detected RHEL/Fedora/CentOS system."
        echo "Installing pkg-config..."
        sudo yum install pkg-config
        ;;
    alpine)
        echo "Detected Alpine Linux system."
        echo "Installing pkg-config..."
        sudo apk add pkg-config
        ;;
    *)
        echo "Unsupported distribution: $DISTRO"
        exit 1
        ;;
esac

# Install required tools and libraries.
case $DISTRO in
    fedora|rhel)
        echo "Detected Fedora/RHEL system."
        echo "Installing Development Tools..."
        sudo dnf groupinstall "Development Tools"
        ;;
    ubuntu|debian)
        echo "Detected Ubuntu/Debian system."
        echo "Installing build-essential..."
        sudo apt install build-essential
        ;;
    alpine)
        echo "Detected Alpine Linux system."
        echo "Installing alpine-sdk and bsd-compat-headers..."
        sudo apk add alpine-sdk bsd-compat-headers
        ;;
    *)
        echo "Unsupported distribution: $DISTRO"
        exit 1
        ;;
esac

# Function to check if pip3 is installed
check_pip3() {
    if ! command -v pip3 &> /dev/null; then
        echo "pip3 could not be found. Please install it first."
        exit 1
    fi
}

check_pip3

echo "Installing meson and ninja using pip3..."
pip3 install meson ninja

# Install `pyelftools`
case $DISTRO in
    fedora)
        echo "Detected Fedora system."
        echo "Installing pyelftools..."
        sudo dnf install python-pyelftools
        ;;
    rhel|centos)
        echo "Detected RHEL/CentOS system."
        check_pip3
        echo "Installing pyelftools using pip3..."
        pip3 install pyelftools
        ;;
    ubuntu|debian)
        echo "Detected Ubuntu/Debian system."
        echo "Installing python3-pyelftools..."
        sudo apt install python3-pyelftools
        ;;
    alpine)
        echo "Detected Alpine Linux system."
        echo "Installing py3-elftools..."
        sudo apk add py3-elftools
        ;;
    *)
        echo "Unsupported distribution: $DISTRO"
        exit 1
        ;;
esac

# Install libraries for handling NUMA.
case $DISTRO in
    rhel|fedora)
        echo "Detected RHEL/Fedora system."
        echo "Installing numactl-devel..."
        sudo yum install numactl-devel
        ;;
    ubuntu|debian)
        echo "Detected Ubuntu/Debian system."
        echo "Installing libnuma-dev..."
        sudo apt install libnuma-dev
        ;;
    alpine)
        echo "Detected Alpine Linux system."
        echo "Installing numactl-dev..."
        sudo apk add numactl-dev
        ;;
    *)
        echo "Unsupported distribution: $DISTRO"
        exit 1
        ;;
esac

# Install pip3.
case $DISTRO in
    rhel|centos)
        echo "Detected RHEL/CentOS system."
        echo "Installing pip3..."
        yum install epel-release
        yum install python-pip
        ;;
    fedora)
        echo "Detected Fedora system"
        echo "Installing pip3..."
        sudo dnf install python3-pip
    ;;
    ubuntu|debian)
        echo "Detected Ubuntu/Debian system."
        echo "Installing pip3..."
        sudo apt install python3-pip
        ;;
    alpine)
        echo "Detected Alpine Linux system."
        echo "Installing pip3..."
        sudo apk add numactl-dev
        ;;
    *)
        echo "Unsupported distribution: $DISTRO"
        exit 1
        ;;
esac

# Install meson
sudo pip3 install meson

# Install ninja
sudo pip3 install ninja

# Store the project root path.
ROOT_PATH=$(pwd)

# Download and compile DPDK from Source.
DPDK_VERSION="23.11"
echo "Downloading and compiling DPDK $DPDK_VERSION from Source..."

# Download from Source
echo "Downloading DPDK..."
wget http://fast.dpdk.org/rel/dpdk-$DPDK_VERSION.tar.xz
tar xJf dpdk-$DPDK_VERSION.tar.xz

# Compile DPDK
echo "Compiling DPDK..."
cd dpdk-$DPDK_VERSION
meson setup build
cd build
ninja
sudo meson install
sudo ldconfig

# Go back to the project root path.
cd $ROOT_PATH

# Download and compile libbpf from Source
LIBBPF_REPOSITORY_URL="https://github.com/libbpf/libbpf.git"
echo "Downloading and compiling libbpf from Source..."
git clone $LIBBPF_REPOSITORY_URL
cd libbpf/src
sudo make
