from microSDR.dsp.fft_overlap_save import OverlapSave
import numpy as np

N = 16384
V = 4

os = OverlapSave(N, V)

# Create a test signal
x = np.arange(N*5, dtype=np.float32)

blocks = list(os.process(x))

print("Generated blocks:", len(blocks))
print("Block shape:", blocks[0].shape)
print("First block first 10 samples:", blocks[0][:10])
print("First block last 10 samples:", blocks[0][-10:])
print("Second block first 10 samples:", blocks[1][:10])
print("Second block last 10 samples:", blocks[1][-10:])
print("Third block first 10 samples:", blocks[2][:10])
print("Third block last 10 samples:", blocks[2][-10:])
