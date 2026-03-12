import sys
import numpy as np
from PyQt6 import QtWidgets

import rx888_dsp
from sdr_window import SdrWindow

FFT_SIZE = 16384
SAMPLE_RATE = "64.8M"

from rx888_dsp import PyStreamManager


# ------------------------------------------------------------
# Total power in dBFS (same math as before)
# ------------------------------------------------------------
def total_power_dbfs(mag, N):
    power = mag**2
    P_total = power.sum()
    P_fs = (N**2) / 4.0
    return 10 * np.log10(P_total / P_fs + 1e-20)


# ------------------------------------------------------------
# Main application
# ------------------------------------------------------------
def main():
    # Create hardware stream + FFT pipeline
    # device = PyRx888Device()              # <-- GUI controls use this
    stream = PyStreamManager(SAMPLE_RATE) # <-- DSP pipeline uses this

    fft = rx888_dsp.PyFftPipeline(FFT_SIZE)

    # Start hardware
    stream.start()
    global running
    running = True

    # Precompute frequency axis (normalized 0..0.5)
    freqs = np.linspace(0, 0.5, fft.spectrum_len())

    # Qt application
    app = QtWidgets.QApplication(sys.argv)
    win = SdrWindow(stream)   # pass stream so GUI can call set_vga(), set_attenuator()
    win.show()

    try:
        while running:
            try:
                # Read raw i16 samples (2 bytes per sample)
                raw = stream.read_samples(FFT_SIZE * 2)
                s = np.frombuffer(raw, dtype='<i2')

                # Rust FFT → NumPy array
                mag = fft.process(raw)
                mag_db = 20 * np.log10(mag + 1e-12)

                # Total power
                total_db = total_power_dbfs(mag, FFT_SIZE)

                # Update GUI
                win.update_spectrum(freqs, mag_db)
                win.update_power(total_db)

                # Keep GUI responsive
                QtWidgets.QApplication.processEvents()
            except Exception:
                break


    except KeyboardInterrupt:
        pass

    stream.stop()
    print("Stopped.")

    # app.exec()


if __name__ == "__main__":
    main()