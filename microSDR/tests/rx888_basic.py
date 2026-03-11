import rx888_dsp
s = rx888_dsp.PyStreamManager()
s.start()
data = s.read_samples(16384)
s.stop()