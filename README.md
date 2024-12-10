# Minimal example of PyO3-logger swallowing exceptions

To reproduce:

```bash
source .venv/bin/activate
maturin develop
python example.py
# And press Ctrl+C partway through running it

```


Feel free to edit `example.py` to e.g. use `sleepy=True` rather than `sleepy=False` and change the log level.

## The error

You get a nice Python stack trace:

```
^CTraceback (most recent call last):
  File "/usr/lib/python3.12/logging/__init__.py", line 1790, in isEnabledFor
    def isEnabledFor(self, level):

KeyboardInterrupt
```

but this exception is eaten rather than re-raising it. It is not available to `py.check_signals()` nor is it available to Python code after the Rust code returns.
