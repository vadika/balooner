#!/bin/bash

# Function to start a QEMU VM
start_vm() {
    local vm_name=$1
    local memory=$2
    local qmp_socket="/tmp/${vm_name}.sock"
    
    qemu-system-x86_64 \
        -name $vm_name \
        -m $memory \
        -nographic \
        -enable-kvm \
        -cpu host \
        -smp 2 \
        -qmp unix:$qmp_socket,server,nowait \
        -monitor none \
        -serial none \
        -drive file=/dev/zero,format=raw,media=disk \
        &
    
    echo "Started $vm_name with $memory MB of memory, QMP socket: $qmp_socket"
}

# Start three VMs
start_vm "vm1" 1024
start_vm "vm2" 2048
start_vm "vm3" 1536

echo "All VMs started. Press Enter to shut them down."
read

# Shutdown all QEMU processes
killall qemu-system-x86_64

echo "All VMs shut down."
