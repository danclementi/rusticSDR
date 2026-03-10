import threading

class CircularBuffer:
    def __init__(self, size: int):
        self.size = size
        self.buffer = bytearray(size)
        self.write_pos = 0
        self.read_pos = 0
        self.count = 0  # <-- amount of data stored
        self.lock = threading.Lock()
        self.data_available = threading.Event()

    def write(self, data: bytes):
        with self.lock:
            n = len(data)

            # If writer outruns reader, overwrite old data
            if n > self.size:
                # Only keep the last 'size' bytes
                data = data[-self.size:]
                n = len(data)

            # If not enough room, drop oldest data
            while self.count + n > self.size:
                # Drop one byte at a time
                self.read_pos = (self.read_pos + 1) % self.size
                self.count -= 1

            end = self.write_pos + n

            if end <= self.size:
                self.buffer[self.write_pos:end] = data
            else:
                first = self.size - self.write_pos
                self.buffer[self.write_pos:] = data[:first]
                self.buffer[:end % self.size] = data[first:]

            self.write_pos = end % self.size
            self.count += n
            self.data_available.set()

    def read(self, n: int) -> bytes:
        with self.lock:
            if self.count == 0:
                self.data_available.clear()
                return b""

            to_read = min(n, self.count)
            end = self.read_pos + to_read

            if end <= self.size:
                chunk = bytes(self.buffer[self.read_pos:end])
            else:
                first = self.size - self.read_pos
                chunk = bytes(self.buffer[self.read_pos:]) + bytes(self.buffer[:end % self.size])

            self.read_pos = end % self.size
            self.count -= to_read

            return chunk
        
    def status(self):
        """
        Returns (count, capacity, read_pos, write_pos).
        Useful for monitoring producer/consumer balance.
        """
        with self.lock:
            return self.count, self.size, self.read_pos, self.write_pos