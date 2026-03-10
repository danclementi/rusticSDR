import numpy as np


class OverlapSave:
    """
    Borgerding-style overlap-save block generator.

    Parameters
    ----------
    N : int
        FFT size.
    V : int
        Overlap factor. N must be divisible by V.
    """
    def __init__(self, N: int, V: int, window: bool = True):
        if N % V != 0:
            raise ValueError(f"N ({N}) must be divisible by V ({V})")

        self.N = N
        self.V = V
        self.M = N // V
        self.L = N - self.M

        # last M samples from previous block
        self.prev = np.zeros(self.M, dtype=np.float32)
        # input samples that weren't enough for a full step yet
        self.pending = np.zeros(0, dtype=np.float32)

        self.window = np.hanning(N).astype(np.float32) if window else None

    def process_fft(self, x: np.ndarray):
        """
        Accumulates samples across calls and yields FFTs of length N.
        """
        # 1. append new samples to pending buffer
        if self.pending.size == 0:
            buf = x
        else:
            buf = np.concatenate((self.pending, x))

        idx = 0
        n = len(buf)

        # 2. process as many full steps of length L as possible
        while idx + self.L <= n:
            new = buf[idx:idx + self.L]
            block = np.concatenate((self.prev, new))
            self.prev = block[-self.M:]
            idx += self.L

            if self.window is not None:
                block = block * self.window

            spec = np.fft.rfft(block)
            yield spec

        # 3. keep leftover samples for the next call
        self.pending = buf[idx:]