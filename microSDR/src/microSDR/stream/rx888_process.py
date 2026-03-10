import subprocess
import sys
from typing import Generator, Optional


class RX888Process:
    """
    Launches the rx888_stream.exe process and streams raw I/Q samples.
    """

    def __init__(
        self,
        exe_path: str,
        firmware_path: str,
        sample_rate: int = 50_000_000,
        gain: int = 10,
        attenuation: int = 0,
        packet_size: int = 131072,
        num_transfers: int = 32,
    ):
        self.exe_path = exe_path
        self.firmware_path = firmware_path
        self.sample_rate = sample_rate
        self.gain = gain
        self.attenuation = attenuation
        self.packet_size = packet_size
        self.num_transfers = num_transfers
        self.proc: Optional[subprocess.Popen] = None

    def start(self):
        """
        Launch the Rust streamer as a subprocess.
        """
        cmd = [
            self.exe_path,
            "-f", self.firmware_path,
            "--sample-rate", str(self.sample_rate),
            "--gain", str(self.gain),
            "--attenuation", str(self.attenuation),
            "--packet-size", str(self.packet_size),
            "--num-transfers", str(self.num_transfers),
            "--output", "-"
        ]

        self.proc = subprocess.Popen(
            cmd,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            bufsize=0  # unbuffered
        )

        if self.proc.stdout is None:
            raise RuntimeError("Failed to open stdout pipe from rx888_stream.exe")

    def stop(self):
        """
        Stop the subprocess cleanly.
        """
        if self.proc is not None:
            self.proc.terminate()
            self.proc.wait(timeout=2)
            self.proc = None

    def stream_bytes(self, chunk_size: int = 4096) -> Generator[bytes, None, None]:
        """
        Generator that yields raw I/Q bytes from the device.
        """
        if self.proc is None:
            raise RuntimeError("RX888Process not started. Call start() first.")

        stdout = self.proc.stdout

        while True:
            data = stdout.read(chunk_size)
            if not data:
                break
            yield data