# 0.2 (unreleased)

This is a fairly large release with API fixes and improvements, bug fixes, and much better test
coverage and documentation.

## API changes

* We now return a dedicated error type instead of abusing `std::io::Error`.
* `PacketIdentifier` was renamed to `Pid`. It now avoids the illegal value 0, wraps around automatically, and can be hashed.
* `Publish.qos` and `Publish.pid` have been merged together, avoiding accidental illegal combinations.
* `Connect.password` and `Connect.will.payload` can now contain binary data.
* The `Protocol` enum doesn't carry extra data anymore.
* All public structs/enum/functions are now (re)exported from the crate root, and the rest has been made private.
* The letter-casing of packet types is more consistent.
* Packet subtypes can be converted to `Packet` using `.into()`.

## Other changes

* Much improved documentation. See it with `cargo doc --open`.
* More thorough unittesting, including exhaustive and random value ranges testing.
* Lots of corner-case bugfixes, particularly when decoding partial or corrupted data.
* The minimum rust version is now 1.32.
* Raised `mqttrs`'s bus factor to 2 ;)

# 0.1.4 (2019-09-16)

* Fix issue #8: Decoding an incomplete packet still consumes bytes from the buffer.
