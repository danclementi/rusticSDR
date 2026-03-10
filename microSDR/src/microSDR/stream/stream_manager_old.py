import threading
import time
from typing import Optional

from .rx888_process import RX888Process
from .circular_buffer import CircularBuffer

import numpy as np

from microSDR.dsp.real_io import bytes_to_float32_real
from microSDR.dsp.fft_overlap_save import OverlapSave


class StreamManager:
    """
    Manages the RX888Process and a background thread that writes incoming
    samples into a circular buffer. DSP code reads from the buffer.
    """

    def __init__(
        self,
        exe_path: str,
        firmware_path: str,
        buffer_size: int = 16 * 1024 * 1024,  # 16 MB default
        sample_rate: int = 50_000_000,
        gain: int = 10,
        attenuation: int = 0,
    ):
        self.buffer = CircularBuffer(buffer_size)

        self.rx = RX888Process(
            exe_path=exe_path,
            firmware_path=firmware_path,
            sample_rate=sample_rate,
            gain=gain,
            attenuation=attenuation,
        )

        self.thread: Optional[threading.Thread] = None
        self.running = False

    def _writer_loop(self):
        print("Writer loop started")   # <--- ADD THIS

        for chunk in self.rx.stream_bytes(4096):
            # print("Got chunk:", len(chunk))   # <--- ADD THIS TOO
            # print("Got chunk:", len(chunk), "WRITER buffer id:", id(self.buffer)) 
            if not self.running:
                break
            self.buffer.write(chunk)

        print("Writer loop exiting")   # <--- OPTIONAL
        self.running = False

    def start(self):
        """
        Start the RX888 streamer and the writer thread.
        """
        self.rx.start()
        self.running = True

        self.thread = threading.Thread(
            target=self._writer_loop,
            daemon=True
        )
        self.thread.start()

    def stop(self):
        """
        Stop the writer thread and the RX888 process.
        """
        self.running = False
        if self.thread is not None:
            self.thread.join(timeout=2)
            self.thread = None

        self.rx.stop()

    def read_samples(self, n_bytes: int) -> bytes:
        """
        Read raw bytes from the circular buffer.
        """
        return self.buffer.read(n_bytes)

    def wait_for_data(self, timeout=None) -> bool:
        """
        Block until new data arrives.
        """
        return self.buffer.wait_for_data(timeout)
    
    def fft_stream(self, N: int = 16384, V: int = 4, chunk_bytes: int = 32768):
        from microSDR.dsp.real_io import bytes_to_float32_real
        from microSDR.dsp.fft_overlap_save import OverlapSave

        # print("fft_stream starting")
        os = OverlapSave(N=N, V=V)

        while True:
            data = self.buffer.read(chunk_bytes)
            if data is None:
                # print("FFT: read -> None")
                continue
            if len(data) == 0:
                # print("FFT: read -> empty bytes")
                continue

            # print(f"FFT: got {len(data)} bytes")

            x = bytes_to_float32_real(data)

            for spec in os.process_fft(x):
                # print("FFT: produced spectrum")
                yield spec

    # def fft_stream(self, N: int = 16384, V: int = 4, chunk_bytes: int = 4096):
    #     """
    #     Generator: yields FFT spectra from the live RX-888 stream.

    #     N: FFT size
    #     V: overlap factor
    #     chunk_bytes: how many bytes to read from the circular buffer per iteration
    #     """
    #     os = OverlapSave(N=N, V=V)

    #     print("FFT_STREAM buffer id:", id(self.buffer))

    #     while True:
    #         # 1. Read raw bytes from the circular buffer
    #         data = self.buffer.read(chunk_bytes)
    #         if data is None or len(data) == 0:
    #             continue  # or break if you want to stop on empty

    #         # 2. Convert bytes -> float32 real samples
    #         x = bytes_to_float32_real(data)

    #         # 3. Feed into overlap-save FFT engine
    #         for spec in os.process_fft(x):
    #             yield spec                