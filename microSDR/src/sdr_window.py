from PyQt6 import QtWidgets, QtCore
import pyqtgraph as pg
import time

class SdrWindow(QtWidgets.QWidget):
    def __init__(self, stream):
        super().__init__()

        self.stream = stream
        self.setWindowTitle("microSDR Control Panel")
        self.resize(1000, 600)

        # ------------------------------------------------------------
        # Spectrum Plot
        # ------------------------------------------------------------
        self.plot = pg.PlotWidget(title="Spectrum")
        self.plot.showGrid(x=True, y=True)
        self.plot.setLabel('left', 'Amplitude', units='dB')
        self.plot.setLabel('bottom', 'Frequency', units='Hz')
        self.curve = self.plot.plot(pen='y')

        # ------------------------------------------------------------
        # VGA Gain Slider (AD8370)
        # ------------------------------------------------------------
        self.vga_slider = QtWidgets.QSlider(QtCore.Qt.Orientation.Horizontal)
        self.vga_slider.setRange(1, 127)
        self.vga_slider.setValue(40)
        self.vga_slider.valueChanged.connect(self.on_vga_changed)

        # ------------------------------------------------------------
        # Attenuator Dropdown (DAT31)
        # ------------------------------------------------------------
        self.att_dropdown = QtWidgets.QComboBox()
        self.att_dropdown.addItems(["0 dB", "10 dB", "20 dB", "30 dB"])
        self.att_dropdown.currentIndexChanged.connect(self.on_att_changed)

        # ------------------------------------------------------------
        # Power Display
        # ------------------------------------------------------------
        self.power_label = QtWidgets.QLabel("Power: --- dBFS")
        self.power_label.setStyleSheet("font-size: 16px; font-weight: bold;")

        # ------------------------------------------------------------
        # Layout
        # ------------------------------------------------------------
        controls = QtWidgets.QHBoxLayout()
        controls.addWidget(QtWidgets.QLabel("VGA"))
        controls.addWidget(self.vga_slider)
        controls.addWidget(QtWidgets.QLabel("ATT"))
        controls.addWidget(self.att_dropdown)
        controls.addWidget(self.power_label)

        layout = QtWidgets.QVBoxLayout()
        layout.addWidget(self.plot)
        layout.addLayout(controls)
        self.setLayout(layout)

    # ------------------------------------------------------------
    # Manual control callbacks
    # ------------------------------------------------------------
    def on_vga_changed(self, code):
        reg = 0x80 | code  # high-gain mode, amp ON
        self.stream.set_vga(reg)

    def on_att_changed(self, idx):
        att_values = [0, 10, 20, 30]
        att = att_values[idx]
        self.stream.set_attenuator(att)

    # ------------------------------------------------------------
    # Update functions called from your streaming loop
    # ------------------------------------------------------------
    def update_spectrum(self, freqs, mags_db):
        self.curve.setData(freqs, mags_db)

    def update_power(self, dbfs):
        self.power_label.setText(f"Power: {dbfs:.1f} dBFS")

    def closeEvent(self, event):
        global running
        running = False   # <-- stop the main loop FIRST
        time.sleep(0.1)   # <-- give it a moment to halt the stream

        try:
            self.stream.stop()
            event.accept()
        except Exception as e:
            print("Error stopping stream:", e)
        event.accept()

