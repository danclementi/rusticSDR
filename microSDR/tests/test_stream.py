import time
import numpy as np
from rx888_dsp import PyStreamManager

N = 1024
V = 4
CHUNK = 4096

sm = PyStreamManager(N, V, CHUNK)

print("StreamManager started")

try:
    while True:
        frame = sm.poll_frame()
        # if frame is not None:
        #     spec = np.array(frame, dtype=np.complex64)
        #     mag = 20 * np.log10(np.abs(spec) + 1e-12)
        #     peak = np.argmax(mag[:N//2])
        #     print("Peak bin:", peak)

        if frame is not None:
            # Convert the raw complex frame into a NumPy array
            spec = np.array(frame, dtype=np.complex64)

            # Print the first few samples
            print("First 10 samples:", spec[:10])
            break

        if int(time.time()) % 2 == 0:
            print("Stats:", sm.stats())

except KeyboardInterrupt:
    sm.stop()