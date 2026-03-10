import numpy as np

def bytes_to_float32_real(data: bytes) -> np.ndarray:
    """
    Convert raw int16 little-endian real samples into float32 normalized samples.
    """
    # Interpret as int16
    arr = np.frombuffer(data, dtype=np.int16)

    # Normalize to [-1, 1]
    return arr.astype(np.float32) / 32768.0