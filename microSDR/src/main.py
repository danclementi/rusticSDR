import numpy as np
import matplotlib.pyplot as plt
import rx888_dsp
import time

FFT_SIZE = 16384
SAMPLE_RATE = "64.8M"

def main():
    # Create hardware stream + FFT pipeline
    stream = rx888_dsp.PyStreamManager(SAMPLE_RATE)
    fft = rx888_dsp.PyFftPipeline(FFT_SIZE)

    # Start hardware
    stream.start()

    # Prepare matplotlib figure
    fig, ax = plt.subplots()
    x = np.linspace(0, 0.5, fft.spectrum_len())  # normalized frequency (0..0.5)
    line, = ax.plot(x, np.zeros_like(x))

    ax.set_title("Real-Time FFT (Rust DSP)")
    ax.set_xlabel("Normalized Frequency")
    ax.set_ylabel("Magnitude")
    ax.set_ylim(0, 50000)  # adjust as needed

    plt.ion()
    plt.show()

    try:
        while True:
            if not plt.fignum_exists(fig.number):
                break
        
            # Read raw i16 samples (2 bytes per sample)
            # raw = stream.read_samples(FFT_SIZE * 2)

            raw = stream.read_samples(16384 * 2)
            s = np.frombuffer(raw, dtype='<i2')

            # print("std:", np.std(s))
            # print("mean:", np.mean(s))
            # print("max:", np.max(s), "min:", np.min(s))

            # s = np.frombuffer(raw, dtype='<i2')  # little-endian int16
            # print("min:", s.min(), "max:", s.max(), "mean:", s.mean())
            # print("first 16 samples:", s[:16])

            # Rust FFT → NumPy array
            mag = fft.process(raw)
            mag_db = 20 * np.log10(mag + 1e-12)


            # Update plot
            line.set_ydata(mag_db)
            ax.set_ylim(np.max(mag_db) - 80, np.max(mag_db) + 5)

            fig.canvas.draw()
            fig.canvas.flush_events()

            # Small pacing delay
            time.sleep(0.01)

    except KeyboardInterrupt:
        pass

    stream.stop()
    print("Stopped.")

if __name__ == "__main__":
    main()