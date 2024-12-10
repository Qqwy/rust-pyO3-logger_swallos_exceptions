import logging
import incrementer

logging.basicConfig(format="%(levelname)s: %(message)s", level=logging.DEBUG)

client = incrementer.IncrementerClient()

# Ctrl+C does not work here:
client.sum(100000, 200000, False)

# It will work (most of the time) here, because the chance of interrupting the sleep is much higher:
# client.sum(100000, 200000, True)

