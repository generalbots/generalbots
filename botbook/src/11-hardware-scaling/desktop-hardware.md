# Desktop & Workstation Hardware Guide

A detailed guide crossing high-performance AI models with hardware availability and pricing (prices in BRL).

> **Important Note:** Proprietary models like **Claude Opus 4.5**, **GPT-5.2**, and **Gemini 3 Pro** represent the cutting edge of Cloud AI. For **Local AI**, we focus on efficiently running models that approximate this power using **MoE (Mixture of Experts)** technology, specifically **GLM-4.7**, **DeepSeek**, and **OSS120B-GPT**.

## AI Model Scaling for Local Hardware

Mapping mentioned top-tier models to their local "runnable" equivalents.

| Citation Model | Real Status | Local Equivalent (GPU) | Size (Params) |
| :--- | :--- | :--- | :--- |
| **Claude Opus 4.5** | API Only | **GLM-4.7** (MoE) | ~9B to 16B (Highly Efficient) |
| **GPT-5.2** | API Only | **DeepSeek-V3** (MoE) | ~236B (Single RTX High RAM) |
| **Gemini 3 Pro** | API Only | **OSS120B-GPT** (MoE) | ~120B (Single RTX) |
| **GPT-4o** | API Only | DeepSeek-V2-Lite | ~16B (efficient) |
 
### Recommended Models (GGUF Links & File Sizes)

**GLM-4-9B Chat (9B parameters):**
- **Q4_K_M:** [bartowski/glm-4-9b-chat-GGUF](https://huggingface.co/bartowski/glm-4-9b-chat-GGUF) - 5.7GB file, needs 8GB VRAM
- **Q6_K:** Same link - 8.26GB file, needs 10GB VRAM  
- **Q8_0:** Same link - 9.99GB file, needs 12GB VRAM

**DeepSeek-V3 (671B total, 37B active MoE):**
- **Q2_K:** [bartowski/deepseek-ai_DeepSeek-V3-GGUF](https://huggingface.co/bartowski/deepseek-ai_DeepSeek-V3-GGUF) - ~280GB file, needs 32GB VRAM
- **Q4_K_M:** Same link - 409GB file, needs 48GB VRAM (2x RTX 3090)
- **Q6_K:** Same link - 551GB file, needs 64GB VRAM (impossible on consumer GPUs)

**Mistral Large 2407 (123B parameters):**
- **Q2_K:** [bartowski/Mistral-Large-Instruct-2407-GGUF](https://huggingface.co/bartowski/Mistral-Large-Instruct-2407-GGUF) - ~50GB file, needs 24GB VRAM
- **Q4_K_M:** Same link - ~75GB file, needs 32GB VRAM (2x RTX 3060)
- **Q6_K:** Same link - ~95GB file, needs 48GB VRAM (2x RTX 3090)

## Compatibility Matrix (GPU x Model x Quantization)

Defining how well each GPU runs the listed models, focusing on "Best Performance".

**Quantization Legend:**
*   **Q4_K_M:** The "Gold Standard" for home use. Good balance of speed and intelligence.
*   **Q5_K_M / Q6_K:** High quality, slower, requires more VRAM.
*   **Q8_0:** Near perfection (FP16 equivalent), but very heavy.
*   **Offload CPU:** Model fits in system RAM, not VRAM (slow).

| GPU | VRAM | System RAM | **GLM-4-9B** <br>*(Q4_K_M: 5.7GB)* | **DeepSeek-V3** <br>*(Q2_K: 280GB)* | **Mistral Large** <br>*(Q4_K_M: 75GB)* |
| :--- | :--- | :--- | :--- | :--- | :--- |
| **RTX 3050** | 8 GB | 16 GB | **Q8_0** (Perfect) | CPU Offload (Very Slow) | Impossible |
| **RTX 3060** | 12 GB | 32 GB | **Q8_0** (Instant) | CPU Offload (Slow) | CPU Offload (Slow) |
| **RTX 4060 Ti** | 16 GB | 32 GB | **Q8_0** (Overkill) | CPU Offload (Slow) | CPU Offload (Slow) |
| **RTX 3090** | 24 GB | 64 GB | **Q8_0** (Dual Models) | CPU Offload (Usable) | **Q2_K** (Fits!) |
| **2x RTX 3090** | 48 GB | 128 GB | N/A | **Q4_K_M** (Good) | **Q4_K_M** (Perfect) |
| **4x RTX 3090** | 96 GB | 256 GB | N/A | **Q6_K** (Excellent) | **Q6_K** (Excellent) |

## Market Pricing & Minimum Specs
*Approximate prices in BRL (R$).*

| GPU | Used Price (OLX/ML) | New Price (ML) | Min System RAM | RAM Cost (Approx.) | Min CPU | **GLM-4-9B** | **DeepSeek-V3** | **Mistral Large** |
| :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- |
| **RTX 3050 (8GB)** | R$ 750 - R$ 950 | R$ 1.400 - R$ 1.600 | 16 GB (DDR4) | R$ 180 (Used) | i5-10400 / Ryzen 3600 | ✅ **Q8_0** (10GB) | ❌ Too small | ❌ Too small |
| **RTX 3060 (12GB)** | R$ 1.100 - R$ 1.400 | R$ 1.800 - R$ 2.400 | 32 GB (DDR4) | R$ 350 (Used Kit) | Ryzen 5 5600X / i5-12400F | ✅ **Q8_0** (10GB) | ⚠️ CPU offload only | ⚠️ CPU offload only |
| **RTX 4060 Ti (16GB)** | R$ 2.000 - R$ 2.300 | R$ 2.800 - R$ 3.200 | 32 GB (DDR5) | R$ 450 (Used Kit) | Ryzen 7 5700X3D / i5-13400F | ✅ **Q8_0** (10GB) | ⚠️ CPU offload only | ⚠️ CPU offload only |
| **RTX 3070 (8GB)** | R$ 1.200 - R$ 1.500 | N/A | 32 GB (DDR4) | R$ 350 (Used Kit) | Ryzen 7 5800X | ✅ **Q6_K** (8GB) | ❌ Too small | ❌ Too small |
| **RTX 3090 (24GB)** | R$ 3.500 - R$ 4.500 | R$ 10.000+ (Rare) | 64 GB (DDR4/5) | R$ 700 (Kit 32x2) | Ryzen 9 5900X / i7-12700K | ✅ **Q8_0** (10GB) | ⚠️ CPU offload (280GB) | ✅ **Q2_K** (24GB) |
| **RTX 4090 (24GB)** | R$ 9.000 - R$ 11.000 | R$ 12.000 - R$ 15.000 | 64 GB (DDR5) | R$ 900 (Kit 32x2) | Ryzen 9 7950X / i9-13900K | ✅ **Q8_0** (10GB) | ⚠️ CPU offload (280GB) | ✅ **Q2_K** (24GB) |
| **RTX 4080 Super (16GB)** | R$ 6.000 - R$ 7.000 | R$ 7.500 - R$ 9.000 | 64 GB (DDR5) | R$ 900 (Kit 32x2) | Ryzen 9 7900X | ✅ **Q8_0** (10GB) | ⚠️ CPU offload only | ⚠️ CPU offload only |
| **2x RTX 3090 (48GB)** | R$ 7.000 - R$ 9.000 | N/A | 128 GB (DDR4/5) | R$ 1.400 (Kit 64x2) | Ryzen 9 5950X / i9-12900K | ✅ Multiple models | ✅ **Q4_K_M** (409GB) | ✅ **Q4_K_M** (75GB) |

## Technical Analysis & DeepSeek Support

To achieve performance similar to **GLM 4** or **DeepSeek** locally, consider these factors:

### 1. GGUF File Sizes vs VRAM Requirements
**GLM-4-9B (9 billion parameters):**
- Q2_K: 3.99GB file → needs 6GB VRAM
- Q4_K_M: 5.7GB file → needs 8GB VRAM  
- Q6_K: 8.26GB file → needs 10GB VRAM
- Q8_0: 9.99GB file → needs 12GB VRAM

**DeepSeek-V3 (671B total, 37B active MoE):**
- Q2_K: ~280GB file → needs 32GB VRAM (impossible on single consumer GPU)
- Q4_K_M: 409GB file → needs 48GB VRAM (2x RTX 3090 minimum)
- Q6_K: 551GB file → needs 64GB VRAM (3x RTX 3090 or data center)

**Mistral Large 2407 (123B parameters):**
- Q2_K: ~50GB file → needs 24GB VRAM (RTX 3090/4090)
- Q4_K_M: ~75GB file → needs 32GB VRAM (2x RTX 3060 or better)
- Q6_K: ~95GB file → needs 48GB VRAM (2x RTX 3090)

### 2. Reality Check: DeepSeek-V3 Needs Serious Hardware
**DeepSeek-V3** is a 671B parameter MoE model. Even with only 37B active parameters per token, the GGUF files are massive:
- **Minimum viable:** Q2_K at 280GB requires 32GB VRAM (impossible on consumer GPUs)
- **Recommended:** Q4_K_M at 409GB requires 48GB VRAM (2x RTX 3090 = R$ 8.000+)
- **For most users:** Stick to **GLM-4-9B** or **Mistral Large** for local AI

**GLM-4-9B** is the sweet spot:
- Q8_0 (9.99GB) runs perfectly on RTX 3060 12GB
- Near-identical performance to much larger models
- Costs under R$ 2.000 total system cost

### 3. DeepSeek & MoE (Mixture of Experts) in General Bots

**DeepSeek-V2/V3** uses an architecture called **MoE (Mixture of Experts)**. This is highly efficient but requires specific support.

**General Bots Offline Component (llama.cpp):**
The General Bots local LLM component is built on `llama.cpp`, which fully supports MoE models like DeepSeek and Mixtral efficiently.

*   **MoE Efficiency:** Only a fraction of parameters are active for each token generation. DeepSeek-V2 might have 236B parameters total, but only uses ~21B per token.
*   **Running DeepSeek:**
    *   On an **RTX 3060**, you can run **DeepSeek-V2-Lite (16B)** exceptionally well.
    *   It offers performance rivaling much larger dense models.
    *   **Configuration:** Simply select the model in your `local-llm` setup. The internal `llama.cpp` engine handles the MoE routing automatically. No special Flags (`-moe`) are strictly required in recent versions, but ensuring you have the latest `botserver` update guarantees the `llama.cpp` binary supports these architectures.

### 4. Recommended Configurations by Budget

**Entry Level (R$ 2.500 total):**
- **GPU:** RTX 3060 12GB (Used ~R$ 1.300)
- **RAM:** 32 GB DDR4 (~R$ 350)
- **Runs:** GLM-4-9B Q8_0 (perfect), Mistral-7B, Llama-3-8B
- **File sizes:** 10GB models fit comfortably

**Prosumer (R$ 5.000 total):**
- **GPU:** RTX 3090 24GB (Used ~R$ 4.000)  
- **RAM:** 64 GB DDR4 (~R$ 700)
- **Runs:** GLM-4-9B + Mistral Large Q2_K (24GB), multiple models simultaneously
- **File sizes:** Up to 50GB models

**Enterprise (R$ 10.000+):**
- **GPU:** 2x RTX 3090 (48GB total VRAM)
- **RAM:** 128 GB DDR4/5 (~R$ 1.400)
- **Runs:** DeepSeek-V3 Q4_K_M (409GB), Mistral Large Q4_K_M (75GB)
- **File sizes:** 400GB+ models with excellent performance
