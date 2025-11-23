# NVIDIA GPU Setup for LXC Containers

This guide covers setting up NVIDIA GPU passthrough for BotServer running in LXC containers, enabling hardware acceleration for local LLM inference.

## Prerequisites

- NVIDIA GPU (RTX 3060 or better with 12GB+ VRAM recommended)
- NVIDIA drivers installed on the host system
- LXD/LXC installed
- CUDA-capable GPU

## LXD Configuration (Interactive Setup)

When initializing LXD, use these settings:

```bash
sudo lxd init
```

Answer the prompts as follows:
- **Would you like to use LXD clustering?** → `no`
- **Do you want to configure a new storage pool?** → `no` (will create `/generalbots` later)
- **Would you like to connect to a MAAS server?** → `no`
- **Would you like to create a new local network bridge?** → `yes`
- **What should the new bridge be called?** → `lxdbr0`
- **What IPv4 address should be used?** → `auto`
- **What IPv6 address should be used?** → `auto`
- **Would you like the LXD server to be available over the network?** → `no`
- **Would you like stale cached images to be updated automatically?** → `no`
- **Would you like a YAML "lxd init" preseed to be printed?** → `no`

### Storage Configuration
- **Storage backend name:** → `default`
- **Storage backend driver:** → `zfs`
- **Create a new ZFS pool?** → `yes`

## NVIDIA GPU Configuration

### On the Host System

Create a GPU profile and attach it to your container:

```bash
# Create GPU profile
lxc profile create gpu

# Add GPU device to profile
lxc profile device add gpu gpu gpu gputype=physical

# Apply GPU profile to your container
lxc profile add gb-system gpu
```

### Inside the Container

Configure NVIDIA driver version pinning and install drivers:

1. **Pin NVIDIA driver versions** to ensure stability:

```bash
cat > /etc/apt/preferences.d/nvidia-drivers << 'EOF'
Package: *nvidia*
Pin: version 560.35.05-1
Pin-Priority: 1001

Package: cuda-drivers*
Pin: version 560.35.05-1
Pin-Priority: 1001

Package: libcuda*
Pin: version 560.35.05-1
Pin-Priority: 1001

Package: libxnvctrl* 
Pin: version 560.35.05-1
Pin-Priority: 1001

Package: libnv*
Pin: version 560.35.05-1
Pin-Priority: 1001
EOF
```

2. **Install NVIDIA drivers and CUDA toolkit:**

```bash
# Update package lists
apt update

# Install NVIDIA driver and nvidia-smi
apt install -y nvidia-driver nvidia-smi

# Add CUDA repository
wget https://developer.download.nvidia.com/compute/cuda/repos/debian12/x86_64/cuda-keyring_1.1-1_all.deb
dpkg -i cuda-keyring_1.1-1_all.deb

# Install CUDA toolkit
apt-get update
apt-get -y install cuda-toolkit-12-8
apt-get install -y cuda-drivers
```

## Verify GPU Access

After installation, verify GPU is accessible:

```bash
# Check GPU is visible
nvidia-smi

# Should show your GPU with driver version 560.35.05
```

## Configure BotServer for GPU

Update your bot's `config.csv` to use GPU acceleration:

```csv
name,value
llm-server-gpu-layers,35
```

The number of layers depends on your GPU memory:
- **RTX 3060 (12GB):** 20-35 layers
- **RTX 3070 (8GB):** 15-25 layers
- **RTX 4070 (12GB):** 30-40 layers
- **RTX 4090 (24GB):** 50-99 layers

## Troubleshooting

### GPU Not Detected

If `nvidia-smi` doesn't show the GPU:

1. Check host GPU drivers:
   ```bash
   # On host
   nvidia-smi
   lxc config device list gb-system
   ```

2. Verify GPU passthrough:
   ```bash
   # Inside container
   ls -la /dev/nvidia*
   ```

3. Check kernel modules:
   ```bash
   lsmod | grep nvidia
   ```

### Driver Version Mismatch

If you encounter driver version conflicts:

1. Ensure host and container use the same driver version
2. Remove the version pinning file and install matching drivers:
   ```bash
   rm /etc/apt/preferences.d/nvidia-drivers
   apt update
   apt install nvidia-driver-560
   ```

### CUDA Library Issues

If CUDA libraries aren't found:

```bash
# Add CUDA to library path
echo '/usr/local/cuda/lib64' >> /etc/ld.so.conf.d/cuda.conf
ldconfig

# Add to PATH
echo 'export PATH=/usr/local/cuda/bin:$PATH' >> ~/.bashrc
source ~/.bashrc
```

## Custom llama.cpp Compilation

If you need custom CPU/GPU optimizations or specific hardware support, compile llama.cpp from source:

### Prerequisites

```bash
sudo apt update
sudo apt install build-essential cmake git
```

### Compilation Steps

```bash
# Clone llama.cpp repository
git clone https://github.com/ggerganov/llama.cpp
cd llama.cpp

# Create build directory
mkdir build
cd build

# Configure with CUDA support
cmake .. -DLLAMA_CUDA=ON -DLLAMA_CURL=OFF

# Compile using all available cores
make -j$(nproc)
```

### Compilation Options

For different hardware configurations:

```bash
# CPU-only build (no GPU)
cmake .. -DLLAMA_CURL=OFF

# CUDA with specific compute capability
cmake .. -DLLAMA_CUDA=ON -DLLAMA_CUDA_FORCE_COMPUTE=75

# ROCm for AMD GPUs
cmake .. -DLLAMA_HIPBLAS=ON

# Metal for Apple Silicon
cmake .. -DLLAMA_METAL=ON

# AVX2 optimizations for modern CPUs
cmake .. -DLLAMA_AVX2=ON

# F16C for half-precision support
cmake .. -DLLAMA_F16C=ON
```

### After Compilation

```bash
# Copy compiled binary to BotServer
cp bin/llama-server /path/to/botserver-stack/bin/llm/

# Update config.csv to use custom build
llm-server-path,/path/to/botserver-stack/bin/llm/
```

### Benefits of Custom Compilation

- **Hardware-specific optimizations** for your exact CPU/GPU
- **Custom CUDA compute capabilities** for newer GPUs
- **AVX/AVX2/AVX512** instructions for faster CPU inference
- **Reduced binary size** by excluding unused features
- **Support for experimental features** not in releases

## Performance Optimization

### Memory Settings

For optimal LLM performance with GPU:

```csv
name,value
llm-server-gpu-layers,35
llm-server-mlock,true
llm-server-no-mmap,false
llm-server-ctx-size,4096
```

### Multiple GPUs

For systems with multiple GPUs, specify which GPU to use:

```bash
# List available GPUs
lxc profile device add gpu gpu0 gpu gputype=physical id=0
lxc profile device add gpu gpu1 gpu gputype=physical id=1
```

## Benefits of GPU Acceleration

With GPU acceleration enabled:
- **5-10x faster** inference compared to CPU
- **Higher context sizes** possible (8K-32K tokens)
- **Real-time responses** even with large models
- **Lower CPU usage** for other tasks
- **Support for larger models** (13B, 30B parameters)

## Next Steps

- [Installation Guide](./installation.md) - Complete BotServer setup
- [Quick Start](./quick-start.md) - Create your first bot
- [Configuration Reference](../chapter-02/gbot.md) - All GPU-related parameters