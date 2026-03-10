from stream.rx888_process import RX888Process

def main():
    rx = RX888Process(
        exe_path=r"../rx888_stream/target/release/rx888_stream.exe",
        sample_rate=50_000_000,
        gain=10
    )

    rx.start()

    for i, chunk in enumerate(rx.stream_bytes(4096)):
        print(f"Chunk {i}: {len(chunk)} bytes")
        if i == 10:
            break

    rx.stop()

if __name__ == "__main__":
    main()