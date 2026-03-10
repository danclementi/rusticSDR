import numpy as np
from rx888_dsp import PyFftPipeline

N = 1024
V = 4

pipe = PyFftPipeline(N, V)

fs = 1_000_000
f = 100_000
t = np.arange(4096) / fs

# cosine is fine here
samples = 0.5 * np.cos(2 * np.pi * f * t)

# signed 16-bit little-endian
bytes_in = (samples * 32767).astype('<i2').tobytes()

frames = pipe.push_bytes(bytes_in)
print("Frames returned:", len(frames))

if frames:
    spec = frames[0]
    mag = 20 * np.log10(np.abs(spec) + 1e-12)

    # look only at positive frequencies
    half = mag[:N // 2]
    peak_bin = np.argmax(half)
    print("Peak bin (positive half):", peak_bin)