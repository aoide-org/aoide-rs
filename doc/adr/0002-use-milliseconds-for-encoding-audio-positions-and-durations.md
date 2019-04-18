# 2. Use milliseconds for encoding audio positions and durations

Date: 2019-04-18

## Status

Proposed

## Context

Positions and durations in digital audio could be represented using different
measurements:

* time-based, e.g. in seconds or sub-seconds
* sample-based, i.e. in *sample frames* (1 frame = samples from all channels)

The sample-based measurement must always consider complete sample frames with
one sample from each channel (interleaved) to avoid any dependencies on the
actual number of channels.

The value of a sample-based measurement will vary depending on the actual sample
rate while the time-based measurement is independent of the sample rate.

Time-based measurements could be represented by different units. A common
representation are seconds (SI unit) stored with fractional digits, i.e.
a floating-point number or fixed-point representation.

Depending on the use case an integer representation might be sufficient
if sub-seconds are used as the unit, i.e milliseconds or microcseconds.
Milliseconds should provide an acceptable compromise between readability
and precision.

## Decision

We will use time-based measurements for encoding positions and durations in
digital audio streams to avoid dependencies on the actual encoding (sample rate,
number of channels) of the data.

Values will be encoded as floating-point numbers using milliseconds as the unit.

## Consequences

Seconds as a floating-point number are a commonly used for positions and durations.
When importing or exporting data one has to convert between seconds and milliseconds
by multiplying/dividing by 1000.

Since the internal representation uses floating-point the actual unit doesn't really
matter, not considering slight rounding errors caused by binary vs. decimal
floating-point encoding. But if the number of decimal places of the textual
representation is limited or fixed then using milliseconds provides a higher
precision.

Using integers with millisecond precision (e.g. for track durations) will improve
the readability in JSON and is more compact. Floating-point numbers in JSON do not
require a decimal point.