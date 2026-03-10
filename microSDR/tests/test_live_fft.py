import time
import numpy as np
from microSDR.stream.stream_manager import StreamManager

def main():
    sm = StreamManager(
        exe_path="../rx888_stream/target/release/rx888_stream.exe",
        firmware_path="../rx888_stream/SDDC_FX3.img",
        buffer_size=8 * 1024 * 1024,
        sample_rate=64_800_000,
        gain=10,
    )

    sm.start()

    try:
        fft_gen = sm.fft_stream(N=3240, V=5, chunk_bytes=32768)

        for i, spec in enumerate(fft_gen):
            if i % 50 == 0:
                count, cap, r, w = sm.buffer.status()
                print(f"BUFFER: {count}/{cap} bytes  r={r}  w={w}")

            mag = 20 * np.log10(np.abs(spec) + 1e-12)
            # print(f"FFT {i}: range=({mag.min():.1f}, {mag.max():.1f})")
 
        # for i in range(n):
        #     spec = next(fft_gen)  # complex spectrum
        #     mag = 20 * np.log10(np.abs(spec) + 1e-12)
        #     # print(f"FFT {i}: len={len(spec)}, mag range=({mag.min():.1f}, {mag.max():.1f})")
        #     # time.sleep(0.05)

    finally:

        sm.stop()

if __name__ == "__main__":
    main()