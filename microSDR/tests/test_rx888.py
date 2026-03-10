import numpy as np
import matplotlib.pyplot as plt
import matplotlib.animation as animation
from rx888_dsp import PyStreamManager

# ---------------------------------------
# User parameters
# ---------------------------------------
N = 2                 # number of buffers
V = 2                 # vector size
CHUNK_SAMPLES = 16384 # samples per frame
FFT_SIZE = 4096
WATERFALL_HEIGHT = 400

# ---------------------------------------
# Start the SDR stream
# ---------------------------------------
stream = PyStreamManager(N, V, CHUNK_SAMPLES)

# Rolling waterfall buffer
waterfall = np.zeros((WATERFALL_HEIGHT, FFT_SIZE))

# Precompute window
window = np.hanning(CHUNK_SAMPLES)

# ---------------------------------------
# Matplotlib setup
# ---------------------------------------
fig, ax = plt.subplots(figsize=(10, 6))
img = ax.imshow(
    waterfall,
    aspect='auto',
    origin='lower',
    cmap='viridis',
    interpolation='nearest'
)

ax.set_title("Real-Time Waterfall")
ax.set_xlabel("Frequency Bin")
ax.set_ylabel("Time (scrolling)")

# Set a useful dB range
img.set_clim(-60, 0)

# ---------------------------------------
# Frame update function
# ---------------------------------------
def update_frame(_):
    global waterfall

    frame = stream.try_recv_frame()
    if frame is None:
        return [img]

    # Convert to NumPy array
    iq = np.asarray(frame)

    # Apply window
    iq_win = iq * window[:len(iq)]

    # FFT
    fft = np.fft.fftshift(np.fft.fft(iq_win, FFT_SIZE))
    mag = np.abs(fft)

    # Normalize per-frame so noise has contrast
    mag /= np.max(mag) + 1e-12

    # Convert to dB
    db = 20 * np.log10(mag + 1e-12)

    # Scroll waterfall and insert new row
    waterfall = np.roll(waterfall, -1, axis=0)
    waterfall[-1, :] = db


    print("std:", np.std(iq), "max:", np.max(iq), "min:", np.min(iq))

    img.set_data(waterfall)
    return [img]

# ---------------------------------------
# Run animation
# ---------------------------------------
ani = animation.FuncAnimation(
    fig,
    update_frame,
    interval=30,
    blit=True
)

plt.show()

# ---------------------------------------
# Cleanup
# ---------------------------------------
stream.stop()