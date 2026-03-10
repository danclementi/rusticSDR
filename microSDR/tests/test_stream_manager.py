from src.stream.stream_manager import StreamManager
import time

def main():
    sm = StreamManager(
        exe_path=r"../rx888_stream/target/release/rx888_stream.exe",
        firmware_path="../rx888_stream/SDDC_FX3.img",
        buffer_size=4 * 1024 * 1024,
        sample_rate=50_000_000,
        gain=10
    )

    sm.start()

    # Wait for data
    sm.wait_for_data(timeout=10)

    # Read a chunk
    data = sm.read_samples(4096)
    print("Received:", len(data), "bytes")

    sm.stop()

    proc = sm.rx.proc
    if proc is not None:
        print("Return code:", proc.returncode)
        if proc.stderr:
            err = proc.stderr.read().decode(errors="ignore")
            print("stderr:\n", err)

if __name__ == "__main__":
    main()